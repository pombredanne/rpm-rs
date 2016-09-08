# rpm-rs

A parser for [RPM] archives, written in [Rust], using the [nom] parser
library.

_BEWARE_: This code is **incomplete** and probably **terrible**.
I'm using it as a way to teach myself Rust.

For edutainment purposes only.
Causes cancer in laboratory animals. Contains 0% fruit juice.
Not intended for production use. Not intended for experimental use.
Not intended for children. Not intended for adults either. Do not feed to infants under one year of age. Light fuse, get away. ALL-ONE! OK!

[RPM]: http://rpm.org/
[Rust]: http://rust-lang.org/
[nom]: https://github.com/Geal/nom

## RPM file format references

The RPM file format was designed 20+ years ago, when CPUs were 200MHz and packages needed to fit on floppy disks. It has... quirks.

Here are the references I've used in writing this code:

* _Maximum RPM_: http://www.rpm.org/max-rpm/s1-rpm-file-format-rpm-file-format.html
* _rpm.org wiki_: http://rpm.org/wiki/DevelDocs/FileFormat
* _LSB 3.1_: https://refspecs.linuxbase.org/LSB_3.1.0/LSB-Core-generic/LSB-Core-generic/pkgformat.html
