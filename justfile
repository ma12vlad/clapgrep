release := 'false'

appid := if release == 'true' { 'de.leopoldluley.Clapgrep' } else { 'de.leopoldluley.Clapgrep.Devel' }
bin-target := if release == 'true' { 'release' } else { 'debug' }
release-flag := if release == 'true' { '--release' } else { '' }

export APP_ID := appid

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / bin-target / 'clapgrep-gnome'
bin-dst := base-dir / 'bin' / 'clapgrep'

desktop := appid + '.desktop'
desktop-src := 'assets' / desktop
desktop-dst := base-dir / 'share' / 'applications' / desktop

metainfo := 'de.leopoldluley.Clapgrep.metainfo.xml'
metainfo-src := 'assets' / metainfo
metainfo-dst := base-dir / 'share' / 'metainfo' / metainfo

icons-src := 'assets' / 'icons' / 'hicolor'
icons-dst := base-dir / 'share' / 'icons' / 'hicolor'

icon-svg-src := icons-src / 'scalable' / 'apps' / appid + '.svg'
icon-svg-dst := icons-dst / 'scalable' / 'apps' / appid + '.svg'

po-src := 'assets' / 'locale'
po-dst := base-dir / 'share' / 'locale'

default:
  just --list

clean:
  cargo clean

build *args: build-translations
  cargo build --package clapgrep-gnome {{args}} {{release-flag}}

check *args: build
  cargo clippy --all-features {{args}}

run *args: build
  env RUST_BACKTRACE=full cargo run --package clapgrep-gnome {{args}}

ci: setup-flatpak-repos
  echo "skip:" > build-aux/Makefile
  flatpak-builder --keep-build-dirs --disable-updates --build-only --ccache --force-clean flatpak build-aux/{{appid}}.json
  echo Check formatting:
  ./build-aux/fun.sh cargo fmt --all -- --check --verbose
  echo Check code:
  ./build-aux/fun.sh cargo check
  echo Check code with Clippy:
  ./build-aux/fun.sh cargo clippy --workspace --all-targets --all-features -- -D warnings

install: build
  mkdir -p {{po-dst}}
  install -Dm0755 {{bin-src}} {{bin-dst}}
  install -Dm0755 {{desktop-src}} {{desktop-dst}}
  install -Dm0755 {{metainfo-src}} {{metainfo-dst}}
  install -Dm0755 {{icon-svg-src}} {{icon-svg-dst}}
  cp -r {{po-src}} {{po-dst}}

make-makefile target='build-aux/Makefile':
  echo "# This file was generated by 'just make-makefile'" > {{target}}
  echo ".PHONY: install" >> {{target}}
  echo "install:" >> {{target}}
  just -n release={{release}} prefix=/app install 2>&1 | sed 's/^/\t/' | sed 's/\$/$$/g' >> {{target}}

make-cargo-sources:
  python3 build-aux/flatpak-cargo-generator.py ./Cargo.lock -o build-aux/cargo-sources.json

install-flatpak: setup-flatpak-repos make-makefile
  flatpak-builder flatpak-build build-aux/{{appid}}.json --force-clean --install --user

setup-flatpak-repos:
	flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
	flatpak install --or-update --user --noninteractive flathub org.gnome.Platform//47 org.gnome.Sdk//47 org.freedesktop.Sdk.Extension.rust-stable//24.08

gettext:
  xgettext \
    --from-code=UTF-8 \
    --add-comments \
    --keyword=_ \
    --keyword=C_:1c,2 \
    --language=C \
    --output=po/messages.pot \
    --files-from=po/POTFILES
  xtr \
    --omit-header \
    --keywords gettext \
    --keywords gettext_f \
    gnome/src/main.rs >> po/messages.pot
  cat po/LINGUAS | while read lang; do \
    msgmerge -N -U po/$lang.po po/messages.pot; \
    rm -f po/$lang.po~; \
  done

add-translation language:
  msginit -l {{language}}.UTF8 -o po/{{language}}.po -i po/messages.pot

build-translations:
  cat po/LINGUAS | while read lang; do \
    mkdir -p assets/locale/$lang/LC_MESSAGES; \
    msgfmt -o assets/locale/$lang/LC_MESSAGES/{{appid}}.mo po/$lang.po; \
  done

prepare-release:
  just make-cargo-sources
  just release=true make-makefile makefile
  flatpak-builder --force-clean --repo=repo flatpak build-aux/de.leopoldluley.Clapgrep.json
  flatpak run --command=flatpak-builder-lint org.flatpak.Builder repo repo
