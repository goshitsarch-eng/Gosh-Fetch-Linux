Name:           gosh-fetch-cosmic
Version:        2.0.0
Release:        1%{?dist}
Summary:        Modern download manager for Linux (COSMIC)

License:        AGPL-3.0
URL:            https://github.com/goshitsarch-eng/Gosh-Fetch-linux
Source0:        gosh-fetch-%{version}.tar.gz

Requires:       libxkbcommon
Requires:       libwayland-client
Requires:       libinput
Requires:       systemd-libs
Requires:       libseat
Requires:       vulkan-loader

%description
Gosh Fetch is a powerful and modern download manager for Linux with support
for HTTP, HTTPS, and BitTorrent downloads. Built with libcosmic for a native
COSMIC desktop experience.

%prep
%autosetup -n gosh-fetch-%{version}

%install
install -Dm755 target/release/gosh-fetch-cosmic %{buildroot}%{_bindir}/gosh-fetch-cosmic
install -Dm644 gosh-fetch.desktop %{buildroot}%{_datadir}/applications/io.github.gosh.Fetch.desktop
install -Dm644 io.github.gosh.Fetch.metainfo.xml %{buildroot}%{_datadir}/metainfo/io.github.gosh.Fetch.metainfo.xml
install -Dm644 resources/io.github.gosh.Fetch.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/io.github.gosh.Fetch.png

%files
%license LICENSE
%{_bindir}/gosh-fetch-cosmic
%{_datadir}/applications/io.github.gosh.Fetch.desktop
%{_datadir}/metainfo/io.github.gosh.Fetch.metainfo.xml
%{_datadir}/icons/hicolor/256x256/apps/io.github.gosh.Fetch.png

%changelog
* Fri Jan 10 2025 Gosh <gosh@example.com> - 2.0.0-1
- Initial release
