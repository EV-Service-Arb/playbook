{
  inputs = {
    v_flakes.url = "github:valeratrades/v_flakes?ref=v1.6";
  };
  outputs = { self, v_flakes }:
    let
      inherit (v_flakes) flake-utils;
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import v_flakes.default_nixpkgs { inherit system; };
        inherit (import v_flakes.default_nixpkgs { inherit system; config.allowUnfree = true; }) claude-code;
        rust = v_flakes.rs.default_nightly system;
        github = v_flakes.github {
          inherit pkgs;
          pname = "playbook";
          enable = true;
          lfs = true;
          gitignore.extra = "ref/**/recording.*";
          jobs.errors.augment = [{ name = "flake-app"; args.app = "check"; }];
          containerRelease = { registry = "ghcr.io/ev-service-arb"; lfs = false; }; # the image takes only markdown; the media is 17G
        };
        readme = v_flakes.readme-fw {
          inherit pkgs;
          pname = "playbook";
          defaults = true;
          lastSupportedVersion = null;
          rootDir = ./.;
          badges = [ "ci" ];
        };
        combined = v_flakes.utils.combine { inherit rust; modules = [ github readme ]; };

        port = "59082";
        # the member surface; build.rs bakes in skills/service-arb/, structured/{approved,suggested}/ and ref/**/*.md, and nothing else of the tree reaches the image
        playbook_web = (pkgs.makeRustPlatform { rustc = rust; cargo = rust; }).buildRustPackage {
          pname = "playbook_web";
          version = "0.1.0";
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./playbook # a workspace member, so cargo reads its manifest; build.rs inlines cite_check.rs's DECAY_DAYS
              ./playbook_web
              ./skills/service-arb
              ./structured/approved
              ./structured/suggested
              (pkgs.lib.fileset.fileFilter (f: f.hasExt "md") ./ref)
            ];
          };
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          cargoBuildFlags = [ "-p" "playbook_web" ];
          cargoTestFlags = [ "-p" "playbook_web" ];
          nativeBuildInputs = [ pkgs.cmake pkgs.tailwindcss_4 ]; # aws-lc, under reqwest's rustls; the connect pages' stylesheet
          # std's panic locations name its source inside the toolchain, which would pull all of it into the image
          RUSTFLAGS = "--remap-path-prefix=${rust}=/rust";
          disallowedReferences = [ rust ];
          SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"; # the tests build an HTTP client; the image sets its own
        };
        containerStd = v_flakes.container.implement {
          inherit pkgs;
          pname = "playbook_web";
          containers."" = {
            port = pkgs.lib.toInt port;
            mounts = [ "/data" ];
            sqlite = [ "/data/mcp.db" ];
            healthPath = "/health";
            criticality = "normal";
            entrypoint = [ "${playbook_web}/bin/playbook_web" ];
            workingDir = "/data";
            imageEnv = [ "HOME=/data" "PORT=${port}" ];
          };
        };
      in
      {
        apps.help = v_flakes.utils.help {
          inherit pkgs self system;
          about = "`cargo r -p playbook -- help`: call/skool/yt/doc/chatgpt pullers, checks, photo-metadata, eval\n";
        };

        apps.publish = v_flakes.utils.publish pkgs;

        packages = containerStd.packages // { inherit playbook_web; };
        containers = containerStd.containers;

        apps.check = {
          type = "app";
          meta.description = "check calls, check cites, build playbook_web";
          program = "${pkgs.writeShellApplication {
            name = "check";
            runtimeInputs = [ rust pkgs.git ];
            text = "cargo r -q -p playbook -- check calls && cargo r -q -p playbook -- check cites && nix build .#playbook_web --no-link";
          }}/bin/check";
        };

        apps.eval = {
          type = "app";
          meta.description = "answer playbook/eval.json on this tree and on the last release; exits 1 when a case scores lower";
          program = "${pkgs.writeShellApplication {
            name = "eval";
            runtimeInputs = [ rust pkgs.stdenv.cc pkgs.cmake pkgs.tailwindcss_4 pkgs.git claude-code ];
            text = ''cargo r -p playbook -- eval "$@"'';
          }}/bin/eval";
        };

        apps.mcp-sandbox = {
          type = "app";
          meta.description = "your Claude Code without service-arb, to onboard as a new member (--keep resumes)";
          program = "${pkgs.writeShellApplication {
            name = "mcp-sandbox";
            runtimeInputs = [ pkgs.git pkgs.jq pkgs.chromium ]; # claude comes from the caller's PATH
            text = builtins.readFile ./scripts/mcp-sandbox/run.sh;
          }}/bin/mcp-sandbox";
        };

        devShells.default = pkgs.mkShell {
          shellHook = combined.shellHook;
          packages = [ rust pkgs.sqlite pkgs.tailwindcss_4 pkgs.cargo-insta ] ++ combined.enabledPackages;
          env.PORT = port;
        };
      }
    );
}
