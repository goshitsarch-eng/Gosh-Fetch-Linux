Name:           gosh-fetch
Version:        2.0.0
Release:        1%{?dist}
Summary:        Modern download manager for Linux

License:        AGPL-3.0
URL:            https://github.com/goshitsarch-eng/Gosh-Fetch-linux
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gtk4-devel
BuildRequires:  libadwaita-devel
BuildRequires:  dbus-devel
BuildRequires:  openssl-devel

Requires:       gtk4
Requires:       libadwaita

%description
Gosh Fetch is a powerful and modern download manager for Linux with support
for HTTP, HTTPS, and BitTorrent downloads. Built with GTK4 and libadwaita for
a native GNOME experience.

%prep
%autosetup

%build
cargo build --release --package gosh-fetch-gtk

%install
install -Dm755 target/release/gosh-fetch-gtk %{buildroot}%{_bindir}/gosh-fetch-gtk
install -Dm644 gosh-fetch.desktop %{buildroot}%{_datadir}/applications/io.github.gosh.Fetch.desktop
install -Dm644 io.github.gosh.Fetch.metainfo.xml %{buildroot}%{_datadir}/metainfo/io.github.gosh.Fetch.metainfo.xml
install -Dm644 resources/io.github.gosh.Fetch.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/io.github.gosh.Fetch.png

%files
%license LICENSE
%{_bindir}/gosh-fetch-gtk
%{_datadir}/applications/io.github.gosh.Fetch.desktop
%{_datadir}/metainfo/io.github.gosh.Fetch.metainfo.xml
%{_datadir}/icons/hicolor/256x256/apps/io.github.gosh.Fetch.png

%changelog
* Fri Jan 10 2025 Gosh <gosh@example.com> - 2.0.0-1
- Initial release
