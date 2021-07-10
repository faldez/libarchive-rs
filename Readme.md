# libarchive-rs

## Windows
```
vcpkg install libarchive
```

## Linux
```
sudo apt install build-essential \
    cmake \
    llvm \
    clang \
    libarchive-dev \
    libicu-dev \
    nettle-dev \
    libacl1-dev \
    liblzma-dev \
    libzstd-dev \
    liblz4-dev \
    libbz2-dev \
    zlib1g-dev \
    libxml2-dev
```

## macOS
```
brew install icu4c libarchive bzip2 lz4 zlib expat
EXPORT PKG_CONFIG_PATH="/usr/local/opt/icu4c/lib/pkgconfig:/usr/local/opt/libarchive/lib/pkgconfig:/usr/local/opt/zlib/lib/pkgconfig:/usr/local/opt/expat/lib/pkgconfig"
```