{
  description = "os-rs development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

	outputs = { nixpkgs, ... }: {
    devShells.x86_64-linux.default = let
      pkgs = import nixpkgs { system = "x86_64-linux"; };
			overrides.toolchain.channel = "nightly";
		in pkgs.mkShell {
			buildInputs = with pkgs; [
				clang
				llvmPackages.bintools
				rustup
			];
			RUSTC_VERSION = overrides.toolchain.channel;
			LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
			shellHook = ''
				export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
				export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
				rustup update
				rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
				rustup target add x86_64-unknown-none
				rustup component add llvm-tools-preview
				echo -e "git: \n"
				git log --oneline --graph --decorate --all
				echo -e "\n"
				git branch
				'';
			RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') []);
			LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [];
			BINDGEN_EXTRA_CLANG_ARGS =
			(builtins.map (a: ''-I"${a}/include"'') [])
			++ [
				''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
				''-I"${pkgs.glib.dev}/include/glib-2.0"''
				''-I${pkgs.glib.out}/lib/glib-2.0/include/''
			];
		};
  };
}
