{
  inputs = {
    v_flakes.url = "github:valeratrades/v_flakes?ref=v1.6";
  };
  outputs = { self, v_flakes }:
    let
      # the first of this repository's ten in SA_PORT_RANGE: site_conductor's docs/port_range.md
      SA_PORTS = 52830;
      inherit (v_flakes) flake-utils pre-commit-hooks;
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import v_flakes.default_nixpkgs { inherit system; };
        inherit (import v_flakes.default_nixpkgs { inherit system; config.allowUnfree = true; }) claude-code;
        rust = v_flakes.rs.default_nightly system;
        pre-commit-check = pre-commit-hooks.lib.${system}.run (v_flakes.files.preCommit { inherit pkgs; });
        rs = v_flakes.rs {
          inherit pkgs rust;
          pinDeps = true;
          build.enable = false; # playbook_web's build.rs bakes the corpus; the root is a virtual workspace
        };
        cliTools = [ rust pkgs.stdenv.cc pkgs.mold pkgs.git pkgs.pkg-config ]; # .cargo/config.toml links with mold
        cliEnv.PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig"; # openssl-sys, under `playbook`'s adapters
        check = pkgs.writeShellApplication {
          name = "check";
          runtimeInputs = cliTools;
          runtimeEnv = cliEnv;
          text = "cargo r -q -p playbook -- check && nix build .#playbook_web --no-link";
        };
        digested = pkgs.writeShellApplication {
          name = "digested";
          runtimeInputs = cliTools;
          runtimeEnv = cliEnv;
          text = "cargo r -q -p playbook -- check --digested";
        };
        github = v_flakes.github {
          inherit pkgs rs;
          pname = "playbook";
          enable = true;
          lfs = true;
          gitignore.extra = "knowledge/references/**/recording.*\n!.claude/\n.claude/*\n!.claude/CLAUDE.md";
          jobs.errors.augment = [{ name = "check"; program = check; }];
          jobs.warnings.augment = [{ name = "digested"; program = digested; }];
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
        combined = v_flakes.utils.combine { inherit rust; modules = [ rs github readme ]; };

        port = "59082";
        # the member surface; build.rs compiles knowledge/ and bakes in its README.md, structured/ and references/**/*.md, and nothing else of the tree reaches the image
        build_rust = v_flakes.rs.build_nightly system;
        playbook_web = (pkgs.makeRustPlatform { rustc = build_rust; cargo = build_rust; }).buildRustPackage {
          pname = "playbook_web";
          version = "0.1.0";
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./corpus
              ./playbook # a workspace member, so cargo reads its manifest
              ./playbook_web
              ./knowledge/README.md
              ./knowledge/structured
              ./knowledge/log # compiled with the rest, its citations and `resolved:` links; build.rs bakes none of it
              (pkgs.lib.fileset.fileFilter (f: f.hasExt "md") ./knowledge/references)
            ];
          };
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          cargoBuildFlags = [ "-p" "playbook_web" ];
          cargoTestFlags = [ "-p" "playbook_web" ];
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.openssl ]; # ev_lib's sentry turns on reqwest's native-tls
          # std's panic locations name its source inside the toolchain, which would pull all of it into the image
          RUSTFLAGS = "--remap-path-prefix=${build_rust}=/rust";
          disallowedReferences = [ build_rust ];
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
            imageEnv = [ "HOME=/data" "PORT=${port}" "APP_ENV=production" ];
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

        apps.eval = {
          type = "app";
          meta.description = "answer playbook/eval.json on this tree and on the last release, each through a conductor (PANEL_BIN); exits 1 when a case scores lower";
          program = "${pkgs.writeShellApplication {
            name = "eval";
            runtimeInputs = [ rust pkgs.stdenv.cc pkgs.mold pkgs.git claude-code ]; # .cargo/config.toml links with mold
            # each side's playbook_web links openssl: ev_lib's sentry turns on reqwest's native-tls
            text = ''OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include cargo r -p playbook -- eval "$@"'';
          }}/bin/eval";
        };

        apps.mcp-sandbox = {
          type = "app";
          meta.description = "your Claude Code without the service-arb plugin, to onboard as a new member (--keep resumes)";
          program = "${pkgs.writeShellApplication {
            name = "mcp-sandbox";
            runtimeInputs = [ pkgs.git pkgs.jq pkgs.chromium ]; # claude comes from the caller's PATH
            text = builtins.readFile ./scripts/mcp-sandbox/run.sh;
          }}/bin/mcp-sandbox";
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ playbook_web ];
          shellHook = pre-commit-check.shellHook + combined.shellHook + ''
            cp -f ${v_flakes.files.treefmt { inherit pkgs; }} ./.treefmt.toml
          '';
          packages = [ rust pkgs.sqlite ] ++ pre-commit-check.enabledPackages ++ combined.enabledPackages;
          env.PORT = toString SA_PORTS;
        };
      }
    );
}
