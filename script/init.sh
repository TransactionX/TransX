#!/bin/bash

if [[ "$OSTYPE" == "linux-gnu" ]]; then
	set -e
	if [[ $(whoami) == "root" ]]; then
		MAKE_ME_ROOT=
	else
		MAKE_ME_ROOT=sudo
	fi

	if [ -f /etc/redhat-release ]; then
		echo "Redhat Linux detected.(Arch,Ubuntu/Debian, MacOS Support)"
		echo "This OS is not supported with this script at present. Sorry."
		exit 1
	elif [ -f /etc/SuSE-release ]; then
		echo "Suse Linux detected. (Arch,Ubuntu/Debian, MacOS Support)"
		echo "This OS is not supported with this script at present. Sorry."
		exit 1
	elif [ -f /etc/arch-release ]; then
		echo "Arch Linux detected. (Arch,Ubuntu/Debian, MacOS Support)"
		$MAKE_ME_ROOT pacman -Syu --needed --noconfirm cmake gcc openssl-1.0 pkgconf git clang
		export OPENSSL_LIB_DIR="/usr/lib/openssl-1.0";
		export OPENSSL_INCLUDE_DIR="/usr/include/openssl-1.0"
	elif [ -f /etc/mandrake-release ]; then
		echo "Mandrake Linux detected.(Arch,Ubuntu/Debian, MacOS Support)"
		echo "This OS is not supported with this script at present. Sorry."
		exit 1
	elif [ -f /etc/debian_version ]; then
		echo "Ubuntu/Debian Linux detected.(Arch,Ubuntu/Debian, MacOS Support)"
		$MAKE_ME_ROOT apt update
		$MAKE_ME_ROOT apt install -y cmake pkg-config libssl-dev git gcc build-essential git clang libclang-dev
	else
		echo "Unknown Linux distribution.(Arch,Ubuntu/Debian, MacOS Support)"
		echo "This OS is not supported with this script at present. Sorry."
		exit 1
	fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
	set -e
	echo "Mac OS (Darwin) detected.(Arch,Ubuntu/Debian, MacOS Support)"

	if ! which brew >/dev/null 2>&1; then
		/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
	fi

	brew update
	brew install openssl cmake llvm
elif [[ "$OSTYPE" == "freebsd"* ]]; then
	echo "FreeBSD detected.(Arch,Ubuntu/Debian, MacOS Support)"
	echo "This OS is not supported with this script at present. Sorry."
	exit 1
else
	echo "Unknown operating system.(Arch,Ubuntu/Debian, MacOS Support)"
	echo "This OS is not supported with this script at present. Sorry."
	exit 1
fi

if ! which rustup >/dev/null 2>&1; then
	curl https://sh.rustup.rs -sSf | sh -s -- -y
	source ~/.cargo/env
  rustup install nightly-2020-03-09
  rustup default nightly-2020-03-09-x86_64-unknown-linux-gnu
  rustup target add wasm32-unknown-unknown --toolchain nightly-2020-03-09-x86_64-unknown-linux-gnu
else
	rustup update
	rustup default nightly-2020-03-09-x86_64-unknown-linux-gnu
	rustup target add wasm32-unknown-unknown --toolchain nightly-2020-03-09-x86_64-unknown-linux-gnu
fi


if [[ "$1" == "--cn" ]]; then
	g=$(mktemp -d)
	git clone https://github.com.cnpmjs.org/TransactionX/TransX.git "$g"
	pushd "$g"
	cargo install --force --path ./bin/node/cli #substrate
	popd
else
	g=$(mktemp -d)
	git clone https://github.com/TransactionX/TransX.git "$g"
	pushd "$g"
	cargo install --force --path ./bin/node/cli #substrate
	popd
fi


echo "Great !!! "
echo "TransX Installed"
echo "Run source ~/.cargo/env now to update environment"
echo "TransX --help  for more"