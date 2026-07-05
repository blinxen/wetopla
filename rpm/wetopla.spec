# There are no tests yet
%bcond_with check

Name:           wetopla
Version:        0.1.0
Release:        1%{?dist}
Summary:         Weekly todo planer
# Apache-2.0 OR MIT
# MIT
License:        (Apache-2.0 OR MIT) AND MIT
URL:            https://github.com/blinxen/%{name}
Source:         %{url}/archive/%{version}/%{name}-%{version}.tar.gz

BuildRequires:  cargo-rpm-macros >= 24

%description
Weekly todo planer

%files
%license LICENSE
%license LICENSE.dependencies
%doc README.md
%{_bindir}/wetopla

%prep
%autosetup -n %{name}-%{version_no_tilde} -p1
%cargo_prep

%generate_buildrequires
%cargo_generate_buildrequires

%build
%{cargo_license_summary}
%{cargo_license} > LICENSE.dependencies
%cargo_build

%install
%cargo_install

%if %{with check}
%check
%cargo_test
%endif

%changelog
* Sun Jul 05 2026 blinxen <h-k-81@hotmail.com> - 0.1.0-1
- Initial package
