#!/bin/bash
set -e

TOKEN="$1"
REPO_URL="$2"
LABEL="$3"

cat <<EOF
#cloud-config

# Configure Nushell and Just Sources
apt:
  sources:
    llvm:
      source: |
        deb http://apt.llvm.org/\$RELEASE/ llvm-toolchain-\$RELEASE main
        deb-src http://apt.llvm.org/\$RELEASE/ llvm-toolchain-\$RELEASE main
      key: |
        -----BEGIN PGP PUBLIC KEY BLOCK-----
        Version: GnuPG v1.4.12 (GNU/Linux)

        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
        bXb5c6gCHESH5PXwPU4jQEE7Ib9J6sbk7ZT2Mw==
        =j+4q
        -----END PGP PUBLIC KEY BLOCK-----

    fury:
      source: "deb https://apt.fury.io/nushell/ /"
      key: |
        -----BEGIN PGP PUBLIC KEY BLOCK-----

        xjMEZ7lpChYJKwYBBAHaRw8BAQdACJu5JZBTrTr+eDGx1lt59RRioe2PtVrW1ZFS
        pwXLRCfNHG51c2hlbGwgPG5vcmVwbHlAbnVzaGVsbC5zaD7CkwQTFgoAOxYhBPeT
        m3UPoXiYMtZ6gJ51X8NlvwDVBQJnuWkKAhsDBQsJCAcCAiICBhUKCQgLAgQWAgMB
        Ah4HAheAAAoJEJ51X8NlvwDVKggBALlmjR0qdrL/HpKTWyudPmqMw6RI5yc1mfFU
        RW/GzEVzAP9QKbLQmfmDEX+8S1zm0X6c4u8Nmn91gLcCqWaIKf9vA844BGe5aQoS
        CisGAQQBl1UBBQEBB0APSM89abiQkfJ4AgqXCUPMo7WVzR0v3vb1WPVRRx5IAwMB
        CAfCeAQYFgoAIBYhBPeTm3UPoXiYMtZ6gJ51X8NlvwDVBQJnuWkKAhsMAAoJEJ51
        X8NlvwDVipkA/jH+pxNYws5lzKYzwcjq8gEHGqmv0rYNXgCzHT880qB4AQCHgpTY
        IJcK9nNo8My4ke4kE0Yr2JE92qBn7uP8FkK7DA==
        =I+gW
        -----END PGP PUBLIC KEY BLOCK-----
    prebuilt-mpr:
      source: "deb https://proget.makedeb.org prebuilt-mpr \$RELEASE"
      key: |
        -----BEGIN PGP PUBLIC KEY BLOCK-----
        Version: BCPG C# v1.8.1.0

        mQINBGJslcEBEACWT0Kphz3bmIO7s3BRW8Rw2m36YI0lDQmdkZy9F6Aa8+Ov9b6+
        44hAvPYfm0fjOiGxq264v9Av5/26vcrDYsPKupaQs6EjT0NjcZ2A47KLpeJGzbzt
        74KP+YyKm/oqW8Md8jb8GBRq0dnFoLOOA6bvHzP7TzANAKFIEZQ0m4GO/AI6Y9ap
        09Q+gZramLl9OCoKRhnmC7QiJO9PSLLvBJOWk3pORpMU5GK5df2RYgpC6d0laFe4
        KmmqcmuU7PgGAJ9C0Fy9i4WEEPdIrJ1Sg0PNnCloLGxFYg1he3gkF2GnTHAU05Ih
        yc1anMv4MDh5m3+Em3laEnR16lMPPi/veLvdyGJ61f1I8cih391afvHloN2J3Z+j
        0sZN7BZthIWkoqhd6DnZ+XGwJoM3lAsPaH3uHsNtquWIqEOO0UBndIOmrcVd5QyY
        1tAJBQHl9XDRirIfsbFxESevANTkc6DCe7VPLADK/nTqwVrrruOZAIXrEwnoWTFj
        xxl7lW+zswiVJWcpYIMQClCAgdq0g7bLs3UvPn4cUYYLTXRp+3LDps7hwmmoEYCD
        bsp4exVPS9Whzu2/6ojR4pwJcXdvXsnPsqcQGwwd/pNaJrjC/BmWbQBZaBspfj2X
        AtnhObFJkB49UWr+cFBIcoJAUEL16WyAf/o/1K8n1299Cc1QeUgBg1F41wARAQAB
        tBNjb250YWN0QG1ha2VkZWIub3JniQIlBBMBAgAPApsDBYJibJXBBYkAAAAAAAoJ
        EE/p8sQ9lCig3esQAIXM6ib4PJqKbAd5enhPBkAquZzLIsdaNg3f5dqan5BptIGM
        8EOS82Urlw7PACLlqf8lkM+0riU1SXGuEig+TFHHeYE9xYL9JguLnT+Iumfe/0YM
        y/26VNadQJv581eJUnRfOBdhUFsCwksLE28qq8SKDq9a8n719TgPngTZNaD0zCNB
        Tk90cwa7oRfUQza9IMn/7AvjvkpuwpsyZeSMj4RNL+1PaqgSgF1W94YWbXGO0e+g
        fyp+r8WH0rYEUbl51FxGw5vXV9hGKe2Xbo40TAIv//l//8LeluA3P4nzPnr1pGaq
        iqsIsVTfluC8OC0uEFTnsMCiyEUnP7LlCJc5dZeA+JfSuiwUI117cZ5TQr8uRY9L
        zHiwKJkkrg2XNxwjQ20BZASd4HOHW8NcD/MULUL3d+pQ+lzbqcHLqT69YJdfVnPa
        PEAsjHEauDbwgHyhy2pa2l43GcvecxlZTIXgKH+gGO+G2vC3vMFVcPLN0JWbHEkD
        Tyiht2rMcIaEGXqzigc/S9czkOzsUuFdlWlBslBxJ6/yJYj5kvpSyGRJIe0dKZMp
        yYDQQmO0+OuA75BUveDK9v9msA25NX8Ynus7tNh1LdlSnaJv6JYEIEhWDVBEkPmQ
        TKGNsT2a5xrLry8pS3XJYThitIc8G7/ngzVOejsZ94G4OmYuCQxK97Hf/aHu
        =Vi1D
        -----END PGP PUBLIC KEY BLOCK-----
    docker:
      source: "deb https://download.docker.com/linux/ubuntu \$RELEASE stable"
      key: |
        -----BEGIN PGP PUBLIC KEY BLOCK-----

        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
        YT90qFF93M3v01BbxP+EIY2/9tiIPbrd
        =0YYh
        -----END PGP PUBLIC KEY BLOCK-----


# Install dev-dependencies, just, nu, and CLI utilities
packages:
  - pkg-config
  - build-essential
  - libssl-dev
  - libclang-dev
  - protobuf-compiler
  - nushell
  - just
  - curl
  - jq
  - git
  - docker-ce
  - docker-ce-cli
  - containerd.io
  - docker-buildx-plugin
  - docker-compose-plugin

# Initialize the runner
runcmd:
  - apt install -y clang-format clang-tidy clang-tools clang clangd libc++-dev libc++1 libc++abi-dev libc++abi1 libclang-dev libclang1 liblldb-dev libllvm-ocaml-dev libomp-dev libomp5 lld lldb llvm-dev llvm-runtime llvm python3-clang
  - useradd -m runner
  - cd /home/runner
  - su runner -c '
      curl -O -L https://github.com/actions/runner/releases/download/v2.324.0/actions-runner-linux-x64-2.324.0.tar.gz &&
      tar xzf ./actions-runner-linux-x64-2.324.0.tar.gz &&
      ./config.sh --url $REPO_URL --token $TOKEN --labels $LABEL --unattended &&
      ./run.sh
    '
EOF
