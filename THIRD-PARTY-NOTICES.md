# Third-party notices

Orca Input Viewer is MIT licensed — see [LICENSE](LICENSE). It links and bundles the
third-party software below. Most of it is permissively licensed and needs nothing more
than this record; one component sets conditions, and they are spelled out first.

## libusb — LGPL-2.1-or-later

Every adapter, serial and device read goes through [libusb](https://libusb.info), which is
compiled into the application by the `vendored` feature of `rusb` (`libusb1-sys` 0.7.0).
libusb is free software under the GNU Lesser General Public License, version 2.1 or later,
and the copy used here is unmodified.

The library is linked in statically, so LGPL-2.1 section 6 applies: you are entitled to
replace the libusb inside this application with a modified one.

- Upstream source: <https://github.com/libusb/libusb>
- Build against a libusb you control by removing `features = ["vendored"]` from the `rusb`
  dependency in `src-tauri/Cargo.toml`, which makes the build link the system shared
  library instead. A patched static build can be substituted the same way, with a
  `[patch]` entry for `libusb1-sys`.
- The application's own object files are available on request: mlputterman@gmail.com

<details>
<summary>Full text of the GNU Lesser General Public License, version 2.1</summary>

```text
		  GNU LESSER GENERAL PUBLIC LICENSE
		       Version 2.1, February 1999

 Copyright (C) 1991, 1999 Free Software Foundation, Inc.
 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

[This is the first released version of the Lesser GPL.  It also counts
 as the successor of the GNU Library Public License, version 2, hence
 the version number 2.1.]

			    Preamble

  The licenses for most software are designed to take away your
freedom to share and change it.  By contrast, the GNU General Public
Licenses are intended to guarantee your freedom to share and change
free software--to make sure the software is free for all its users.

  This license, the Lesser General Public License, applies to some
specially designated software packages--typically libraries--of the
Free Software Foundation and other authors who decide to use it.  You
can use it too, but we suggest you first think carefully about whether
this license or the ordinary General Public License is the better
strategy to use in any particular case, based on the explanations below.

  When we speak of free software, we are referring to freedom of use,
not price.  Our General Public Licenses are designed to make sure that
you have the freedom to distribute copies of free software (and charge
for this service if you wish); that you receive source code or can get
it if you want it; that you can change the software and use pieces of
it in new free programs; and that you are informed that you can do
these things.

  To protect your rights, we need to make restrictions that forbid
distributors to deny you these rights or to ask you to surrender these
rights.  These restrictions translate to certain responsibilities for
you if you distribute copies of the library or if you modify it.

  For example, if you distribute copies of the library, whether gratis
or for a fee, you must give the recipients all the rights that we gave
you.  You must make sure that they, too, receive or can get the source
code.  If you link other code with the library, you must provide
complete object files to the recipients, so that they can relink them
with the library after making changes to the library and recompiling
it.  And you must show them these terms so they know their rights.

  We protect your rights with a two-step method: (1) we copyright the
library, and (2) we offer you this license, which gives you legal
permission to copy, distribute and/or modify the library.

  To protect each distributor, we want to make it very clear that
there is no warranty for the free library.  Also, if the library is
modified by someone else and passed on, the recipients should know
that what they have is not the original version, so that the original
author's reputation will not be affected by problems that might be
introduced by others.

  Finally, software patents pose a constant threat to the existence of
any free program.  We wish to make sure that a company cannot
effectively restrict the users of a free program by obtaining a
restrictive license from a patent holder.  Therefore, we insist that
any patent license obtained for a version of the library must be
consistent with the full freedom of use specified in this license.

  Most GNU software, including some libraries, is covered by the
ordinary GNU General Public License.  This license, the GNU Lesser
General Public License, applies to certain designated libraries, and
is quite different from the ordinary General Public License.  We use
this license for certain libraries in order to permit linking those
libraries into non-free programs.

  When a program is linked with a library, whether statically or using
a shared library, the combination of the two is legally speaking a
combined work, a derivative of the original library.  The ordinary
General Public License therefore permits such linking only if the
entire combination fits its criteria of freedom.  The Lesser General
Public License permits more lax criteria for linking other code with
the library.

  We call this license the "Lesser" General Public License because it
does Less to protect the user's freedom than the ordinary General
Public License.  It also provides other free software developers Less
of an advantage over competing non-free programs.  These disadvantages
are the reason we use the ordinary General Public License for many
libraries.  However, the Lesser license provides advantages in certain
special circumstances.

  For example, on rare occasions, there may be a special need to
encourage the widest possible use of a certain library, so that it becomes
a de-facto standard.  To achieve this, non-free programs must be
allowed to use the library.  A more frequent case is that a free
library does the same job as widely used non-free libraries.  In this
case, there is little to gain by limiting the free library to free
software only, so we use the Lesser General Public License.

  In other cases, permission to use a particular library in non-free
programs enables a greater number of people to use a large body of
free software.  For example, permission to use the GNU C Library in
non-free programs enables many more people to use the whole GNU
operating system, as well as its variant, the GNU/Linux operating
system.

  Although the Lesser General Public License is Less protective of the
users' freedom, it does ensure that the user of a program that is
linked with the Library has the freedom and the wherewithal to run
that program using a modified version of the Library.

  The precise terms and conditions for copying, distribution and
modification follow.  Pay close attention to the difference between a
"work based on the library" and a "work that uses the library".  The
former contains code derived from the library, whereas the latter must
be combined with the library in order to run.

		  GNU LESSER GENERAL PUBLIC LICENSE
   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

  0. This License Agreement applies to any software library or other
program which contains a notice placed by the copyright holder or
other authorized party saying it may be distributed under the terms of
this Lesser General Public License (also called "this License").
Each licensee is addressed as "you".

  A "library" means a collection of software functions and/or data
prepared so as to be conveniently linked with application programs
(which use some of those functions and data) to form executables.

  The "Library", below, refers to any such software library or work
which has been distributed under these terms.  A "work based on the
Library" means either the Library or any derivative work under
copyright law: that is to say, a work containing the Library or a
portion of it, either verbatim or with modifications and/or translated
straightforwardly into another language.  (Hereinafter, translation is
included without limitation in the term "modification".)

  "Source code" for a work means the preferred form of the work for
making modifications to it.  For a library, complete source code means
all the source code for all modules it contains, plus any associated
interface definition files, plus the scripts used to control compilation
and installation of the library.

  Activities other than copying, distribution and modification are not
covered by this License; they are outside its scope.  The act of
running a program using the Library is not restricted, and output from
such a program is covered only if its contents constitute a work based
on the Library (independent of the use of the Library in a tool for
writing it).  Whether that is true depends on what the Library does
and what the program that uses the Library does.
  
  1. You may copy and distribute verbatim copies of the Library's
complete source code as you receive it, in any medium, provided that
you conspicuously and appropriately publish on each copy an
appropriate copyright notice and disclaimer of warranty; keep intact
all the notices that refer to this License and to the absence of any
warranty; and distribute a copy of this License along with the
Library.

  You may charge a fee for the physical act of transferring a copy,
and you may at your option offer warranty protection in exchange for a
fee.

  2. You may modify your copy or copies of the Library or any portion
of it, thus forming a work based on the Library, and copy and
distribute such modifications or work under the terms of Section 1
above, provided that you also meet all of these conditions:

    a) The modified work must itself be a software library.

    b) You must cause the files modified to carry prominent notices
    stating that you changed the files and the date of any change.

    c) You must cause the whole of the work to be licensed at no
    charge to all third parties under the terms of this License.

    d) If a facility in the modified Library refers to a function or a
    table of data to be supplied by an application program that uses
    the facility, other than as an argument passed when the facility
    is invoked, then you must make a good faith effort to ensure that,
    in the event an application does not supply such function or
    table, the facility still operates, and performs whatever part of
    its purpose remains meaningful.

    (For example, a function in a library to compute square roots has
    a purpose that is entirely well-defined independent of the
    application.  Therefore, Subsection 2d requires that any
    application-supplied function or table used by this function must
    be optional: if the application does not supply it, the square
    root function must still compute square roots.)

These requirements apply to the modified work as a whole.  If
identifiable sections of that work are not derived from the Library,
and can be reasonably considered independent and separate works in
themselves, then this License, and its terms, do not apply to those
sections when you distribute them as separate works.  But when you
distribute the same sections as part of a whole which is a work based
on the Library, the distribution of the whole must be on the terms of
this License, whose permissions for other licensees extend to the
entire whole, and thus to each and every part regardless of who wrote
it.

Thus, it is not the intent of this section to claim rights or contest
your rights to work written entirely by you; rather, the intent is to
exercise the right to control the distribution of derivative or
collective works based on the Library.

In addition, mere aggregation of another work not based on the Library
with the Library (or with a work based on the Library) on a volume of
a storage or distribution medium does not bring the other work under
the scope of this License.

  3. You may opt to apply the terms of the ordinary GNU General Public
License instead of this License to a given copy of the Library.  To do
this, you must alter all the notices that refer to this License, so
that they refer to the ordinary GNU General Public License, version 2,
instead of to this License.  (If a newer version than version 2 of the
ordinary GNU General Public License has appeared, then you can specify
that version instead if you wish.)  Do not make any other change in
these notices.

  Once this change is made in a given copy, it is irreversible for
that copy, so the ordinary GNU General Public License applies to all
subsequent copies and derivative works made from that copy.

  This option is useful when you wish to copy part of the code of
the Library into a program that is not a library.

  4. You may copy and distribute the Library (or a portion or
derivative of it, under Section 2) in object code or executable form
under the terms of Sections 1 and 2 above provided that you accompany
it with the complete corresponding machine-readable source code, which
must be distributed under the terms of Sections 1 and 2 above on a
medium customarily used for software interchange.

  If distribution of object code is made by offering access to copy
from a designated place, then offering equivalent access to copy the
source code from the same place satisfies the requirement to
distribute the source code, even though third parties are not
compelled to copy the source along with the object code.

  5. A program that contains no derivative of any portion of the
Library, but is designed to work with the Library by being compiled or
linked with it, is called a "work that uses the Library".  Such a
work, in isolation, is not a derivative work of the Library, and
therefore falls outside the scope of this License.

  However, linking a "work that uses the Library" with the Library
creates an executable that is a derivative of the Library (because it
contains portions of the Library), rather than a "work that uses the
library".  The executable is therefore covered by this License.
Section 6 states terms for distribution of such executables.

  When a "work that uses the Library" uses material from a header file
that is part of the Library, the object code for the work may be a
derivative work of the Library even though the source code is not.
Whether this is true is especially significant if the work can be
linked without the Library, or if the work is itself a library.  The
threshold for this to be true is not precisely defined by law.

  If such an object file uses only numerical parameters, data
structure layouts and accessors, and small macros and small inline
functions (ten lines or less in length), then the use of the object
file is unrestricted, regardless of whether it is legally a derivative
work.  (Executables containing this object code plus portions of the
Library will still fall under Section 6.)

  Otherwise, if the work is a derivative of the Library, you may
distribute the object code for the work under the terms of Section 6.
Any executables containing that work also fall under Section 6,
whether or not they are linked directly with the Library itself.

  6. As an exception to the Sections above, you may also combine or
link a "work that uses the Library" with the Library to produce a
work containing portions of the Library, and distribute that work
under terms of your choice, provided that the terms permit
modification of the work for the customer's own use and reverse
engineering for debugging such modifications.

  You must give prominent notice with each copy of the work that the
Library is used in it and that the Library and its use are covered by
this License.  You must supply a copy of this License.  If the work
during execution displays copyright notices, you must include the
copyright notice for the Library among them, as well as a reference
directing the user to the copy of this License.  Also, you must do one
of these things:

    a) Accompany the work with the complete corresponding
    machine-readable source code for the Library including whatever
    changes were used in the work (which must be distributed under
    Sections 1 and 2 above); and, if the work is an executable linked
    with the Library, with the complete machine-readable "work that
    uses the Library", as object code and/or source code, so that the
    user can modify the Library and then relink to produce a modified
    executable containing the modified Library.  (It is understood
    that the user who changes the contents of definitions files in the
    Library will not necessarily be able to recompile the application
    to use the modified definitions.)

    b) Use a suitable shared library mechanism for linking with the
    Library.  A suitable mechanism is one that (1) uses at run time a
    copy of the library already present on the user's computer system,
    rather than copying library functions into the executable, and (2)
    will operate properly with a modified version of the library, if
    the user installs one, as long as the modified version is
    interface-compatible with the version that the work was made with.

    c) Accompany the work with a written offer, valid for at
    least three years, to give the same user the materials
    specified in Subsection 6a, above, for a charge no more
    than the cost of performing this distribution.

    d) If distribution of the work is made by offering access to copy
    from a designated place, offer equivalent access to copy the above
    specified materials from the same place.

    e) Verify that the user has already received a copy of these
    materials or that you have already sent this user a copy.

  For an executable, the required form of the "work that uses the
Library" must include any data and utility programs needed for
reproducing the executable from it.  However, as a special exception,
the materials to be distributed need not include anything that is
normally distributed (in either source or binary form) with the major
components (compiler, kernel, and so on) of the operating system on
which the executable runs, unless that component itself accompanies
the executable.

  It may happen that this requirement contradicts the license
restrictions of other proprietary libraries that do not normally
accompany the operating system.  Such a contradiction means you cannot
use both them and the Library together in an executable that you
distribute.

  7. You may place library facilities that are a work based on the
Library side-by-side in a single library together with other library
facilities not covered by this License, and distribute such a combined
library, provided that the separate distribution of the work based on
the Library and of the other library facilities is otherwise
permitted, and provided that you do these two things:

    a) Accompany the combined library with a copy of the same work
    based on the Library, uncombined with any other library
    facilities.  This must be distributed under the terms of the
    Sections above.

    b) Give prominent notice with the combined library of the fact
    that part of it is a work based on the Library, and explaining
    where to find the accompanying uncombined form of the same work.

  8. You may not copy, modify, sublicense, link with, or distribute
the Library except as expressly provided under this License.  Any
attempt otherwise to copy, modify, sublicense, link with, or
distribute the Library is void, and will automatically terminate your
rights under this License.  However, parties who have received copies,
or rights, from you under this License will not have their licenses
terminated so long as such parties remain in full compliance.

  9. You are not required to accept this License, since you have not
signed it.  However, nothing else grants you permission to modify or
distribute the Library or its derivative works.  These actions are
prohibited by law if you do not accept this License.  Therefore, by
modifying or distributing the Library (or any work based on the
Library), you indicate your acceptance of this License to do so, and
all its terms and conditions for copying, distributing or modifying
the Library or works based on it.

  10. Each time you redistribute the Library (or any work based on the
Library), the recipient automatically receives a license from the
original licensor to copy, distribute, link with or modify the Library
subject to these terms and conditions.  You may not impose any further
restrictions on the recipients' exercise of the rights granted herein.
You are not responsible for enforcing compliance by third parties with
this License.

  11. If, as a consequence of a court judgment or allegation of patent
infringement or for any other reason (not limited to patent issues),
conditions are imposed on you (whether by court order, agreement or
otherwise) that contradict the conditions of this License, they do not
excuse you from the conditions of this License.  If you cannot
distribute so as to satisfy simultaneously your obligations under this
License and any other pertinent obligations, then as a consequence you
may not distribute the Library at all.  For example, if a patent
license would not permit royalty-free redistribution of the Library by
all those who receive copies directly or indirectly through you, then
the only way you could satisfy both it and this License would be to
refrain entirely from distribution of the Library.

If any portion of this section is held invalid or unenforceable under any
particular circumstance, the balance of the section is intended to apply,
and the section as a whole is intended to apply in other circumstances.

It is not the purpose of this section to induce you to infringe any
patents or other property right claims or to contest validity of any
such claims; this section has the sole purpose of protecting the
integrity of the free software distribution system which is
implemented by public license practices.  Many people have made
generous contributions to the wide range of software distributed
through that system in reliance on consistent application of that
system; it is up to the author/donor to decide if he or she is willing
to distribute software through any other system and a licensee cannot
impose that choice.

This section is intended to make thoroughly clear what is believed to
be a consequence of the rest of this License.

  12. If the distribution and/or use of the Library is restricted in
certain countries either by patents or by copyrighted interfaces, the
original copyright holder who places the Library under this License may add
an explicit geographical distribution limitation excluding those countries,
so that distribution is permitted only in or among countries not thus
excluded.  In such case, this License incorporates the limitation as if
written in the body of this License.

  13. The Free Software Foundation may publish revised and/or new
versions of the Lesser General Public License from time to time.
Such new versions will be similar in spirit to the present version,
but may differ in detail to address new problems or concerns.

Each version is given a distinguishing version number.  If the Library
specifies a version number of this License which applies to it and
"any later version", you have the option of following the terms and
conditions either of that version or of any later version published by
the Free Software Foundation.  If the Library does not specify a
license version number, you may choose any version ever published by
the Free Software Foundation.

  14. If you wish to incorporate parts of the Library into other free
programs whose distribution conditions are incompatible with these,
write to the author to ask for permission.  For software which is
copyrighted by the Free Software Foundation, write to the Free
Software Foundation; we sometimes make exceptions for this.  Our
decision will be guided by the two goals of preserving the free status
of all derivatives of our free software and of promoting the sharing
and reuse of software generally.

			    NO WARRANTY

  15. BECAUSE THE LIBRARY IS LICENSED FREE OF CHARGE, THERE IS NO
WARRANTY FOR THE LIBRARY, TO THE EXTENT PERMITTED BY APPLICABLE LAW.
EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT HOLDERS AND/OR
OTHER PARTIES PROVIDE THE LIBRARY "AS IS" WITHOUT WARRANTY OF ANY
KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
PURPOSE.  THE ENTIRE RISK AS TO THE QUALITY AND PERFORMANCE OF THE
LIBRARY IS WITH YOU.  SHOULD THE LIBRARY PROVE DEFECTIVE, YOU ASSUME
THE COST OF ALL NECESSARY SERVICING, REPAIR OR CORRECTION.

  16. IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN
WRITING WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MAY MODIFY
AND/OR REDISTRIBUTE THE LIBRARY AS PERMITTED ABOVE, BE LIABLE TO YOU
FOR DAMAGES, INCLUDING ANY GENERAL, SPECIAL, INCIDENTAL OR
CONSEQUENTIAL DAMAGES ARISING OUT OF THE USE OR INABILITY TO USE THE
LIBRARY (INCLUDING BUT NOT LIMITED TO LOSS OF DATA OR DATA BEING
RENDERED INACCURATE OR LOSSES SUSTAINED BY YOU OR THIRD PARTIES OR A
FAILURE OF THE LIBRARY TO OPERATE WITH ANY OTHER SOFTWARE), EVEN IF
SUCH HOLDER OR OTHER PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH
DAMAGES.

		     END OF TERMS AND CONDITIONS

           How to Apply These Terms to Your New Libraries

  If you develop a new library, and you want it to be of the greatest
possible use to the public, we recommend making it free software that
everyone can redistribute and change.  You can do so by permitting
redistribution under these terms (or, alternatively, under the terms of the
ordinary General Public License).

  To apply these terms, attach the following notices to the library.  It is
safest to attach them to the start of each source file to most effectively
convey the exclusion of warranty; and each file should have at least the
"copyright" line and a pointer to where the full notice is found.

    <one line to give the library's name and a brief idea of what it does.>
    Copyright (C) <year>  <name of author>

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Lesser General Public
    License as published by the Free Software Foundation; either
    version 2.1 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public
    License along with this library; if not, write to the Free Software
    Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA

Also add information on how to contact you by electronic and paper mail.

You should also get your employer (if you work as a programmer) or your
school, if any, to sign a "copyright disclaimer" for the library, if
necessary.  Here is a sample; alter the names:

  Yoyodyne, Inc., hereby disclaims all copyright interest in the
  library `Frob' (a library for tweaking knobs) written by James Random Hacker.

  <signature of Ty Coon>, 1 April 1990
  Ty Coon, President of Vice

That's all there is to it!
```

</details>

## MPL-2.0 components

These are used unmodified. MPL-2.0 is a file-level copyleft: unmodified use imposes no
source-availability obligation on this application, and any file that is modified must be
published under the same license. Source for each component is at the URL listed in the
inventory below; the license text is at <https://www.mozilla.org/MPL/2.0/>.

- cssparser 0.29.6 — MPL-2.0 — https://github.com/servo/rust-cssparser
- cssparser-macros 0.6.1 — MPL-2.0 — https://github.com/servo/rust-cssparser
- dtoa-short 0.3.5 — MPL-2.0 — https://github.com/upsuper/dtoa-short
- option-ext 0.2.0 — MPL-2.0 — https://github.com/soc/option-ext.git
- selectors 0.24.0 — MPL-2.0 — https://github.com/servo/servo
- serialport 4.8.1 — MPL-2.0 — https://github.com/serialport/serialport-rs

## Components with several licenses on offer

Each of these offers a choice. The MIT-family option is the one taken where one exists
(MIT for unescaper and r-efi, MIT-0/CC0 for dunce); the others are used under a permissive
alternative (Apache-2.0 for ryu, CDLA-Permissive-2.0 for webpki-roots). The other licenses
on offer are not taken.

- dunce 1.0.5 — CC0-1.0 OR MIT-0 OR Apache-2.0 — https://gitlab.com/kornelski/dunce
- r-efi 5.3.0 — MIT OR Apache-2.0 OR LGPL-2.1-or-later — https://github.com/r-efi/r-efi
- ryu 1.0.22 — Apache-2.0 OR BSL-1.0 — https://github.com/dtolnay/ryu
- unescaper 0.1.8 — GPL-3.0/MIT — https://github.com/hack-ink/unescaper
- webpki-roots 1.0.5 — CDLA-Permissive-2.0 — https://github.com/rustls/webpki-roots

## Linux AppImage: GTK and WebKitGTK

The AppImage is assembled with linuxdeploy's GTK hook, so it carries the WebKitGTK and GTK
shared libraries it was built against (both LGPL-2.1-or-later, from the distribution used
to build it). They remain separate, replaceable shared objects inside the image, and
nothing in this repository modifies them. Windows uses the WebView2 runtime and macOS the
system WebView — system components, not bundled here.

## No code from the projects credited in the README

Dolphin, Slippi (`slippi-ssbm-asm`), M'Overlay, GP2040-CE and libmelee are credited as
inspiration and as sources of facts — memory addresses, protocols, file formats. No code
from any of them is copied, translated or linked into this application. Should that ever
change, the affected component's license (GPL-2.0-or-later for Dolphin, GPL-3.0 for
`slippi-ssbm-asm`, MIT for the others) would have to be honoured here too.

## Dependency inventory

Generated from `cargo metadata --format-version 1 --locked`; one line per registry crate
in the dependency graph. Licenses are the SPDX expressions the crates declare; their full
texts and copyright notices ship with their sources.

<details>
<summary>510 crates</summary>

```text
adler2 2.0.1 — 0BSD OR MIT OR Apache-2.0 — https://github.com/oyvindln/adler2
aho-corasick 1.1.4 — Unlicense OR MIT — https://github.com/BurntSushi/aho-corasick
alloc-no-stdlib 2.0.4 — BSD-3-Clause — https://github.com/dropbox/rust-alloc-no-stdlib
alloc-stdlib 0.2.2 — BSD-3-Clause — https://github.com/dropbox/rust-alloc-no-stdlib
android_system_properties 0.1.5 — MIT/Apache-2.0 — https://github.com/nical/android_system_properties
anyhow 1.0.100 — MIT OR Apache-2.0 — https://github.com/dtolnay/anyhow
arbitrary 1.4.2 — MIT OR Apache-2.0 — https://github.com/rust-fuzz/arbitrary/
ascii 1.1.0 — Apache-2.0 OR MIT — https://github.com/tomprogrammer/rust-ascii
atk 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
atk-sys 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
atomic-waker 1.1.2 — Apache-2.0 OR MIT — https://github.com/smol-rs/atomic-waker
autocfg 1.5.0 — Apache-2.0 OR MIT — https://github.com/cuviper/autocfg
base64 0.21.7 — MIT OR Apache-2.0 — https://github.com/marshallpierce/rust-base64
base64 0.22.1 — MIT OR Apache-2.0 — https://github.com/marshallpierce/rust-base64
bitflags 1.3.2 — MIT/Apache-2.0 — https://github.com/bitflags/bitflags
bitflags 2.10.0 — MIT OR Apache-2.0 — https://github.com/bitflags/bitflags
block-buffer 0.10.4 — MIT OR Apache-2.0 — https://github.com/RustCrypto/utils
block2 0.6.2 — MIT — https://github.com/madsmtm/objc2
brotli 8.0.2 — BSD-3-Clause AND MIT — https://github.com/dropbox/rust-brotli
brotli-decompressor 5.0.0 — BSD-3-Clause/MIT — https://github.com/dropbox/rust-brotli-decompressor
bumpalo 3.19.1 — MIT OR Apache-2.0 — https://github.com/fitzgen/bumpalo
bytemuck 1.24.0 — Zlib OR Apache-2.0 OR MIT — https://github.com/Lokathor/bytemuck
byteorder 1.5.0 — Unlicense OR MIT — https://github.com/BurntSushi/byteorder
bytes 1.11.0 — MIT — https://github.com/tokio-rs/bytes
cairo-rs 0.18.5 — MIT — https://github.com/gtk-rs/gtk-rs-core
cairo-sys-rs 0.18.2 — MIT — https://github.com/gtk-rs/gtk-rs-core
camino 1.2.2 — MIT OR Apache-2.0 — https://github.com/camino-rs/camino
cargo-platform 0.1.9 — MIT OR Apache-2.0 — https://github.com/rust-lang/cargo
cargo_metadata 0.19.2 — MIT — https://github.com/oli-obk/cargo_metadata
cargo_toml 0.22.3 — Apache-2.0 OR MIT — https://gitlab.com/lib.rs/cargo_toml
cc 1.2.53 — MIT OR Apache-2.0 — https://github.com/rust-lang/cc-rs
cesu8 1.1.0 — Apache-2.0/MIT — https://github.com/emk/cesu8-rs
cfb 0.7.3 — MIT — https://github.com/mdsteele/rust-cfb
cfg-expr 0.15.8 — MIT OR Apache-2.0 — https://github.com/EmbarkStudios/cfg-expr
cfg-if 1.0.4 — MIT OR Apache-2.0 — https://github.com/rust-lang/cfg-if
cfg_aliases 0.2.1 — MIT — https://github.com/katharostech/cfg_aliases
chrono 0.4.43 — MIT OR Apache-2.0 — https://github.com/chronotope/chrono
chunked_transfer 1.5.0 — MIT OR Apache-2.0 — https://github.com/frewsxcv/rust-chunked-transfer
combine 4.6.7 — MIT — https://github.com/Marwes/combine
convert_case 0.4.0 — MIT — https://github.com/rutrum/convert-case
cookie 0.18.1 — MIT OR Apache-2.0 — https://github.com/SergioBenitez/cookie-rs
core-foundation 0.10.0 — MIT OR Apache-2.0 — https://github.com/servo/core-foundation-rs
core-foundation-sys 0.8.7 — MIT OR Apache-2.0 — https://github.com/servo/core-foundation-rs
core-graphics 0.24.0 — MIT OR Apache-2.0 — https://github.com/servo/core-foundation-rs
core-graphics-types 0.2.0 — MIT OR Apache-2.0 — https://github.com/servo/core-foundation-rs
cpufeatures 0.2.17 — MIT OR Apache-2.0 — https://github.com/RustCrypto/utils
crc32c 0.6.8 — Apache-2.0/MIT — https://github.com/zowens/crc32c
crc32fast 1.5.0 — MIT OR Apache-2.0 — https://github.com/srijs/rust-crc32fast
crossbeam-channel 0.5.15 — MIT OR Apache-2.0 — https://github.com/crossbeam-rs/crossbeam
crossbeam-deque 0.8.6 — MIT OR Apache-2.0 — https://github.com/crossbeam-rs/crossbeam
crossbeam-epoch 0.9.18 — MIT OR Apache-2.0 — https://github.com/crossbeam-rs/crossbeam
crossbeam-utils 0.8.21 — MIT OR Apache-2.0 — https://github.com/crossbeam-rs/crossbeam
crypto-common 0.1.7 — MIT OR Apache-2.0 — https://github.com/RustCrypto/traits
cssparser 0.29.6 — MPL-2.0 — https://github.com/servo/rust-cssparser
cssparser-macros 0.6.1 — MPL-2.0 — https://github.com/servo/rust-cssparser
ctor 0.2.9 — Apache-2.0 OR MIT — https://github.com/mmastrac/rust-ctor
darling 0.21.3 — MIT — https://github.com/TedDriggs/darling
darling_core 0.21.3 — MIT — https://github.com/TedDriggs/darling
darling_macro 0.21.3 — MIT — https://github.com/TedDriggs/darling
data-encoding 2.10.0 — MIT — https://github.com/ia0/data-encoding
deranged 0.5.5 — MIT OR Apache-2.0 — https://github.com/jhpratt/deranged
derive_arbitrary 1.4.2 — MIT OR Apache-2.0 — https://github.com/rust-fuzz/arbitrary
derive_more 0.99.20 — MIT — https://github.com/JelteF/derive_more
digest 0.10.7 — MIT OR Apache-2.0 — https://github.com/RustCrypto/traits
dirs 6.0.0 — MIT OR Apache-2.0 — https://github.com/soc/dirs-rs
dirs-sys 0.5.0 — MIT OR Apache-2.0 — https://github.com/dirs-dev/dirs-sys-rs
dispatch 0.2.0 — MIT — http://github.com/SSheldon/rust-dispatch
dispatch2 0.3.0 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
displaydoc 0.2.5 — MIT OR Apache-2.0 — https://github.com/yaahc/displaydoc
dlopen2 0.8.2 — MIT — https://github.com/OpenByteDev/dlopen2
dlopen2_derive 0.4.3 — MIT — https://github.com/OpenByteDev/dlopen2
dpi 0.1.2 — Apache-2.0 AND MIT — https://github.com/rust-windowing/winit
dtoa 1.0.11 — MIT OR Apache-2.0 — https://github.com/dtolnay/dtoa
dtoa-short 0.3.5 — MPL-2.0 — https://github.com/upsuper/dtoa-short
dunce 1.0.5 — CC0-1.0 OR MIT-0 OR Apache-2.0 — https://gitlab.com/kornelski/dunce
dyn-clone 1.0.20 — MIT OR Apache-2.0 — https://github.com/dtolnay/dyn-clone
either 1.15.0 — MIT OR Apache-2.0 — https://github.com/rayon-rs/either
embed-resource 3.0.6 — MIT — https://github.com/nabijaczleweli/rust-embed-resource
embed_plist 1.2.2 — MIT OR Apache-2.0 — https://github.com/nvzqz/embed-plist-rs
equivalent 1.0.2 — Apache-2.0 OR MIT — https://github.com/indexmap-rs/equivalent
erased-serde 0.4.9 — MIT OR Apache-2.0 — https://github.com/dtolnay/erased-serde
errno 0.3.14 — MIT OR Apache-2.0 — https://github.com/lambda-fairy/rust-errno
fastrand 2.3.0 — Apache-2.0 OR MIT — https://github.com/smol-rs/fastrand
fdeflate 0.3.7 — MIT OR Apache-2.0 — https://github.com/image-rs/fdeflate
field-offset 0.3.6 — MIT OR Apache-2.0 — https://github.com/Diggsey/rust-field-offset
filetime 0.2.27 — MIT/Apache-2.0 — https://github.com/alexcrichton/filetime
find-msvc-tools 0.1.8 — MIT OR Apache-2.0 — https://github.com/rust-lang/cc-rs
flate2 1.1.8 — MIT OR Apache-2.0 — https://github.com/rust-lang/flate2-rs
fnv 1.0.7 — Apache-2.0 / MIT — https://github.com/servo/rust-fnv
foreign-types 0.5.0 — MIT/Apache-2.0 — https://github.com/sfackler/foreign-types
foreign-types-macros 0.2.3 — MIT/Apache-2.0 — https://github.com/sfackler/foreign-types
foreign-types-shared 0.3.1 — MIT/Apache-2.0 — https://github.com/sfackler/foreign-types
form_urlencoded 1.2.2 — MIT OR Apache-2.0 — https://github.com/servo/rust-url
futf 0.1.5 — MIT / Apache-2.0 — https://github.com/servo/futf
futures-channel 0.3.31 — MIT OR Apache-2.0 — https://github.com/rust-lang/futures-rs
futures-core 0.3.31 — MIT OR Apache-2.0 — https://github.com/rust-lang/futures-rs
futures-executor 0.3.31 — MIT OR Apache-2.0 — https://github.com/rust-lang/futures-rs
futures-io 0.3.31 — MIT OR Apache-2.0 — https://github.com/rust-lang/futures-rs
futures-macro 0.3.31 — MIT OR Apache-2.0 — https://github.com/rust-lang/futures-rs
futures-sink 0.3.31 — MIT OR Apache-2.0 — https://github.com/rust-lang/futures-rs
futures-task 0.3.31 — MIT OR Apache-2.0 — https://github.com/rust-lang/futures-rs
futures-util 0.3.31 — MIT OR Apache-2.0 — https://github.com/rust-lang/futures-rs
fxhash 0.2.1 — Apache-2.0/MIT — https://github.com/cbreeden/fxhash
gdk 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
gdk-pixbuf 0.18.5 — MIT — https://github.com/gtk-rs/gtk-rs-core
gdk-pixbuf-sys 0.18.0 — MIT — https://github.com/gtk-rs/gtk-rs-core
gdk-sys 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
gdkwayland-sys 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
gdkx11 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
gdkx11-sys 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
generic-array 0.14.7 — MIT — https://github.com/fizyk20/generic-array.git
getrandom 0.1.16 — MIT OR Apache-2.0 — https://github.com/rust-random/getrandom
getrandom 0.2.17 — MIT OR Apache-2.0 — https://github.com/rust-random/getrandom
getrandom 0.3.4 — MIT OR Apache-2.0 — https://github.com/rust-random/getrandom
gio 0.18.4 — MIT — https://github.com/gtk-rs/gtk-rs-core
gio-sys 0.18.1 — MIT — https://github.com/gtk-rs/gtk-rs-core
glib 0.18.5 — MIT — https://github.com/gtk-rs/gtk-rs-core
glib-macros 0.18.5 — MIT — https://github.com/gtk-rs/gtk-rs-core
glib-sys 0.18.1 — MIT — https://github.com/gtk-rs/gtk-rs-core
glob 0.3.3 — MIT OR Apache-2.0 — https://github.com/rust-lang/glob
gobject-sys 0.18.0 — MIT — https://github.com/gtk-rs/gtk-rs-core
gtk 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
gtk-sys 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
gtk3-macros 0.18.2 — MIT — https://github.com/gtk-rs/gtk3-rs
hashbrown 0.12.3 — MIT OR Apache-2.0 — https://github.com/rust-lang/hashbrown
hashbrown 0.16.1 — MIT OR Apache-2.0 — https://github.com/rust-lang/hashbrown
heck 0.4.1 — MIT OR Apache-2.0 — https://github.com/withoutboats/heck
heck 0.5.0 — MIT OR Apache-2.0 — https://github.com/withoutboats/heck
hex 0.4.3 — MIT OR Apache-2.0 — https://github.com/KokaKiwi/rust-hex
html5ever 0.29.1 — MIT OR Apache-2.0 — https://github.com/servo/html5ever
http 1.4.0 — MIT OR Apache-2.0 — https://github.com/hyperium/http
http-body 1.0.1 — MIT — https://github.com/hyperium/http-body
http-body-util 0.1.3 — MIT — https://github.com/hyperium/http-body
httparse 1.10.1 — MIT OR Apache-2.0 — https://github.com/seanmonstar/httparse
httpdate 1.0.3 — MIT OR Apache-2.0 — https://github.com/pyfisch/httpdate
hyper 1.8.1 — MIT — https://github.com/hyperium/hyper
hyper-rustls 0.27.7 — Apache-2.0 OR ISC OR MIT — https://github.com/rustls/hyper-rustls
hyper-util 0.1.19 — MIT — https://github.com/hyperium/hyper-util
iana-time-zone 0.1.64 — MIT OR Apache-2.0 — https://github.com/strawlab/iana-time-zone
iana-time-zone-haiku 0.1.2 — MIT OR Apache-2.0 — https://github.com/strawlab/iana-time-zone
ico 0.4.0 — MIT — https://github.com/mdsteele/rust-ico
icu_collections 2.1.1 — Unicode-3.0 — https://github.com/unicode-org/icu4x
icu_locale_core 2.1.1 — Unicode-3.0 — https://github.com/unicode-org/icu4x
icu_normalizer 2.1.1 — Unicode-3.0 — https://github.com/unicode-org/icu4x
icu_normalizer_data 2.1.1 — Unicode-3.0 — https://github.com/unicode-org/icu4x
icu_properties 2.1.2 — Unicode-3.0 — https://github.com/unicode-org/icu4x
icu_properties_data 2.1.2 — Unicode-3.0 — https://github.com/unicode-org/icu4x
icu_provider 2.1.1 — Unicode-3.0 — https://github.com/unicode-org/icu4x
ident_case 1.0.1 — MIT/Apache-2.0 — https://github.com/TedDriggs/ident_case
idna 1.1.0 — MIT OR Apache-2.0 — https://github.com/servo/rust-url/
idna_adapter 1.2.1 — Apache-2.0 OR MIT — https://github.com/hsivonen/idna_adapter
indexmap 1.9.3 — Apache-2.0 OR MIT — https://github.com/bluss/indexmap
indexmap 2.13.0 — Apache-2.0 OR MIT — https://github.com/indexmap-rs/indexmap
infer 0.19.0 — MIT — https://github.com/bojand/infer
io-kit-sys 0.4.1 — MIT / Apache-2.0 — https://github.com/jtakakura/io-kit-rs
ipnet 2.11.0 — MIT OR Apache-2.0 — https://github.com/krisprice/ipnet
iri-string 0.7.10 — MIT OR Apache-2.0 — https://github.com/lo48576/iri-string
itoa 1.0.17 — MIT OR Apache-2.0 — https://github.com/dtolnay/itoa
javascriptcore-rs 1.1.2 — MIT — https://github.com/tauri-apps/javascriptcore-rs
javascriptcore-rs-sys 1.1.1 — MIT — https://github.com/tauri-apps/javascriptcore-rs
jni 0.21.1 — MIT/Apache-2.0 — https://github.com/jni-rs/jni-rs
jni-sys 0.3.0 — MIT/Apache-2.0 — https://github.com/sfackler/rust-jni-sys
js-sys 0.3.85 — MIT OR Apache-2.0 — https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys
json-patch 3.0.1 — MIT/Apache-2.0 — https://github.com/idubrov/json-patch
jsonptr 0.6.3 — MIT OR Apache-2.0 — https://github.com/chanced/jsonptr
keyboard-types 0.7.0 — MIT OR Apache-2.0 — https://github.com/pyfisch/keyboard-types
kuchikiki 0.8.8-speedreader — MIT — https://github.com/brave/kuchikiki
lazy_static 1.5.0 — MIT OR Apache-2.0 — https://github.com/rust-lang-nursery/lazy-static.rs
libappindicator 0.9.0 — Apache-2.0 OR MIT — https://crates.io/crates/libappindicator
libappindicator-sys 0.9.0 — Apache-2.0 OR MIT — https://crates.io/crates/libappindicator-sys
libc 0.2.180 — MIT OR Apache-2.0 — https://github.com/rust-lang/libc
libloading 0.7.4 — ISC — https://github.com/nagisa/rust_libloading/
libredox 0.1.12 — MIT — https://gitlab.redox-os.org/redox-os/libredox.git
libudev 0.3.0 — MIT — https://github.com/dcuddeback/libudev-rs
libudev-sys 0.1.4 — MIT — https://github.com/dcuddeback/libudev-sys
libusb1-sys 0.7.0 — MIT — https://github.com/a1ien/rusb.git
linux-raw-sys 0.11.0 — Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT — https://github.com/sunfishcode/linux-raw-sys
litemap 0.8.1 — Unicode-3.0 — https://github.com/unicode-org/icu4x
lock_api 0.4.14 — MIT OR Apache-2.0 — https://github.com/Amanieu/parking_lot
log 0.4.29 — MIT OR Apache-2.0 — https://github.com/rust-lang/log
lru-slab 0.1.2 — MIT OR Apache-2.0 OR Zlib — https://github.com/Ralith/lru-slab
mac 0.1.1 — MIT/Apache-2.0 — https://github.com/reem/rust-mac.git
mach2 0.4.3 — BSD-2-Clause OR MIT OR Apache-2.0 — https://github.com/JohnTitor/mach2
markup5ever 0.14.1 — MIT OR Apache-2.0 — https://github.com/servo/html5ever
match_token 0.1.0 — MIT OR Apache-2.0 — https://github.com/servo/html5ever
matches 0.1.10 — MIT — https://github.com/SimonSapin/rust-std-candidates
memchr 2.7.6 — Unlicense OR MIT — https://github.com/BurntSushi/memchr
memoffset 0.9.1 — MIT — https://github.com/Gilnaa/memoffset
mime 0.3.17 — MIT OR Apache-2.0 — https://github.com/hyperium/mime
minisign-verify 0.2.4 — MIT — https://github.com/jedisct1/rust-minisign-verify
miniz_oxide 0.8.9 — MIT OR Zlib OR Apache-2.0 — https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide
mio 1.1.1 — MIT — https://github.com/tokio-rs/mio
muda 0.17.1 — Apache-2.0 OR MIT — https://github.com/amrbashir/muda
ndk 0.9.0 — MIT OR Apache-2.0 — https://github.com/rust-mobile/ndk
ndk-context 0.1.1 — MIT OR Apache-2.0 — https://github.com/rust-windowing/android-ndk-rs
ndk-sys 0.6.0+11769913 — MIT OR Apache-2.0 — https://github.com/rust-mobile/ndk
new_debug_unreachable 1.0.6 — MIT — https://github.com/mbrubeck/rust-debug-unreachable
nix 0.26.4 — MIT — https://github.com/nix-rust/nix
nodrop 0.1.14 — MIT/Apache-2.0 — https://github.com/bluss/arrayvec
ntapi 0.4.2 — Apache-2.0 OR MIT — https://github.com/MSxDOS/ntapi
num-conv 0.1.0 — MIT OR Apache-2.0 — https://github.com/jhpratt/num-conv
num-traits 0.2.19 — MIT OR Apache-2.0 — https://github.com/rust-num/num-traits
num_enum 0.7.5 — BSD-3-Clause OR MIT OR Apache-2.0 — https://github.com/illicitonion/num_enum
num_enum_derive 0.7.5 — BSD-3-Clause OR MIT OR Apache-2.0 — https://github.com/illicitonion/num_enum
objc2 0.6.3 — MIT — https://github.com/madsmtm/objc2
objc2-app-kit 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-cloud-kit 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-core-data 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-core-foundation 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-core-graphics 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-core-image 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-core-text 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-core-video 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-encode 4.1.0 — MIT — https://github.com/madsmtm/objc2
objc2-exception-helper 0.1.1 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-foundation 0.3.2 — MIT — https://github.com/madsmtm/objc2
objc2-io-surface 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-javascript-core 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-osa-kit 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-quartz-core 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-security 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-ui-kit 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
objc2-web-kit 0.3.2 — Zlib OR Apache-2.0 OR MIT — https://github.com/madsmtm/objc2
once_cell 1.21.3 — MIT OR Apache-2.0 — https://github.com/matklad/once_cell
option-ext 0.2.0 — MPL-2.0 — https://github.com/soc/option-ext.git
osakit 0.3.1 — MIT OR Apache-2.0 — https://github.com/mdevils/rust-osakit
pango 0.18.3 — MIT — https://github.com/gtk-rs/gtk-rs-core
pango-sys 0.18.0 — MIT — https://github.com/gtk-rs/gtk-rs-core
parking_lot 0.12.5 — MIT OR Apache-2.0 — https://github.com/Amanieu/parking_lot
parking_lot_core 0.9.12 — MIT OR Apache-2.0 — https://github.com/Amanieu/parking_lot
percent-encoding 2.3.2 — MIT OR Apache-2.0 — https://github.com/servo/rust-url/
phf 0.10.1 — MIT — https://github.com/sfackler/rust-phf
phf 0.11.3 — MIT — https://github.com/rust-phf/rust-phf
phf 0.8.0 — MIT — https://github.com/sfackler/rust-phf
phf_codegen 0.11.3 — MIT — https://github.com/rust-phf/rust-phf
phf_codegen 0.8.0 — MIT — https://github.com/sfackler/rust-phf
phf_generator 0.10.0 — MIT — https://github.com/sfackler/rust-phf
phf_generator 0.11.3 — MIT — https://github.com/rust-phf/rust-phf
phf_generator 0.8.0 — MIT — https://github.com/sfackler/rust-phf
phf_macros 0.10.0 — MIT — https://github.com/sfackler/rust-phf
phf_macros 0.11.3 — MIT — https://github.com/rust-phf/rust-phf
phf_shared 0.10.0 — MIT — https://github.com/sfackler/rust-phf
phf_shared 0.11.3 — MIT — https://github.com/rust-phf/rust-phf
phf_shared 0.8.0 — MIT — https://github.com/sfackler/rust-phf
pin-project-lite 0.2.16 — Apache-2.0 OR MIT — https://github.com/taiki-e/pin-project-lite
pin-utils 0.1.0 — MIT OR Apache-2.0 — https://github.com/rust-lang-nursery/pin-utils
pkg-config 0.3.32 — MIT OR Apache-2.0 — https://github.com/rust-lang/pkg-config-rs
plist 1.8.0 — MIT — https://github.com/ebarnard/rust-plist/
png 0.17.16 — MIT OR Apache-2.0 — https://github.com/image-rs/image-png
potential_utf 0.1.4 — Unicode-3.0 — https://github.com/unicode-org/icu4x
powerfmt 0.2.0 — MIT OR Apache-2.0 — https://github.com/jhpratt/powerfmt
ppv-lite86 0.2.21 — MIT OR Apache-2.0 — https://github.com/cryptocorrosion/cryptocorrosion
precomputed-hash 0.1.1 — MIT — https://github.com/emilio/precomputed-hash
proc-macro-crate 1.3.1 — MIT OR Apache-2.0 — https://github.com/bkchr/proc-macro-crate
proc-macro-crate 2.0.2 — MIT OR Apache-2.0 — https://github.com/bkchr/proc-macro-crate
proc-macro-crate 3.4.0 — MIT OR Apache-2.0 — https://github.com/bkchr/proc-macro-crate
proc-macro-error 1.0.4 — MIT OR Apache-2.0 — https://gitlab.com/CreepySkeleton/proc-macro-error
proc-macro-error-attr 1.0.4 — MIT OR Apache-2.0 — https://gitlab.com/CreepySkeleton/proc-macro-error
proc-macro-hack 0.5.20+deprecated — MIT OR Apache-2.0 — https://github.com/dtolnay/proc-macro-hack
proc-macro2 1.0.105 — MIT OR Apache-2.0 — https://github.com/dtolnay/proc-macro2
quick-xml 0.38.4 — MIT — https://github.com/tafia/quick-xml
quinn 0.11.9 — MIT OR Apache-2.0 — https://github.com/quinn-rs/quinn
quinn-proto 0.11.13 — MIT OR Apache-2.0 — https://github.com/quinn-rs/quinn
quinn-udp 0.5.14 — MIT OR Apache-2.0 — https://github.com/quinn-rs/quinn
quote 1.0.40 — MIT OR Apache-2.0 — https://github.com/dtolnay/quote
r-efi 5.3.0 — MIT OR Apache-2.0 OR LGPL-2.1-or-later — https://github.com/r-efi/r-efi
rand 0.7.3 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
rand 0.8.5 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
rand 0.9.2 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
rand_chacha 0.2.2 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
rand_chacha 0.3.1 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
rand_chacha 0.9.0 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
rand_core 0.5.1 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
rand_core 0.6.4 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
rand_core 0.9.5 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
rand_hc 0.2.0 — MIT/Apache-2.0 — https://github.com/rust-random/rand
rand_pcg 0.2.1 — MIT OR Apache-2.0 — https://github.com/rust-random/rand
raw-window-handle 0.6.2 — MIT OR Apache-2.0 OR Zlib — https://github.com/rust-windowing/raw-window-handle
rayon 1.11.0 — MIT OR Apache-2.0 — https://github.com/rayon-rs/rayon
rayon-core 1.13.0 — MIT OR Apache-2.0 — https://github.com/rayon-rs/rayon
redox_syscall 0.5.18 — MIT — https://gitlab.redox-os.org/redox-os/syscall
redox_syscall 0.7.0 — MIT — https://gitlab.redox-os.org/redox-os/syscall
redox_users 0.5.2 — MIT — https://gitlab.redox-os.org/redox-os/users
ref-cast 1.0.25 — MIT OR Apache-2.0 — https://github.com/dtolnay/ref-cast
ref-cast-impl 1.0.25 — MIT OR Apache-2.0 — https://github.com/dtolnay/ref-cast
regex 1.12.2 — MIT OR Apache-2.0 — https://github.com/rust-lang/regex
regex-automata 0.4.13 — MIT OR Apache-2.0 — https://github.com/rust-lang/regex
regex-syntax 0.8.8 — MIT OR Apache-2.0 — https://github.com/rust-lang/regex
reqwest 0.12.28 — MIT OR Apache-2.0 — https://github.com/seanmonstar/reqwest
ring 0.17.14 — Apache-2.0 AND ISC — https://github.com/briansmith/ring
rusb 0.9.4 — MIT — https://github.com/a1ien/rusb.git
rustc-hash 2.1.1 — Apache-2.0 OR MIT — https://github.com/rust-lang/rustc-hash
rustc_version 0.4.1 — MIT OR Apache-2.0 — https://github.com/djc/rustc-version-rs
rustix 1.1.3 — Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT — https://github.com/bytecodealliance/rustix
rustls 0.23.36 — Apache-2.0 OR ISC OR MIT — https://github.com/rustls/rustls
rustls-pki-types 1.14.0 — MIT OR Apache-2.0 — https://github.com/rustls/pki-types
rustls-webpki 0.103.9 — ISC — https://github.com/rustls/webpki
rustversion 1.0.22 — MIT OR Apache-2.0 — https://github.com/dtolnay/rustversion
ryu 1.0.22 — Apache-2.0 OR BSL-1.0 — https://github.com/dtolnay/ryu
same-file 1.0.6 — Unlicense/MIT — https://github.com/BurntSushi/same-file
schemars 0.8.22 — MIT — https://github.com/GREsau/schemars
schemars 0.9.0 — MIT — https://github.com/GREsau/schemars
schemars 1.2.0 — MIT — https://github.com/GREsau/schemars
schemars_derive 0.8.22 — MIT — https://github.com/GREsau/schemars
scopeguard 1.2.0 — MIT OR Apache-2.0 — https://github.com/bluss/scopeguard
selectors 0.24.0 — MPL-2.0 — https://github.com/servo/servo
semver 1.0.27 — MIT OR Apache-2.0 — https://github.com/dtolnay/semver
serde 1.0.228 — MIT OR Apache-2.0 — https://github.com/serde-rs/serde
serde-untagged 0.1.9 — MIT OR Apache-2.0 — https://github.com/dtolnay/serde-untagged
serde_core 1.0.228 — MIT OR Apache-2.0 — https://github.com/serde-rs/serde
serde_derive 1.0.228 — MIT OR Apache-2.0 — https://github.com/serde-rs/serde
serde_derive_internals 0.29.1 — MIT OR Apache-2.0 — https://github.com/serde-rs/serde
serde_json 1.0.149 — MIT OR Apache-2.0 — https://github.com/serde-rs/json
serde_repr 0.1.20 — MIT OR Apache-2.0 — https://github.com/dtolnay/serde-repr
serde_spanned 0.6.9 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
serde_spanned 1.0.4 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
serde_urlencoded 0.7.1 — MIT/Apache-2.0 — https://github.com/nox/serde_urlencoded
serde_with 3.16.1 — MIT OR Apache-2.0 — https://github.com/jonasbb/serde_with/
serde_with_macros 3.16.1 — MIT OR Apache-2.0 — https://github.com/jonasbb/serde_with/
serialize-to-javascript 0.1.2 — MIT OR Apache-2.0 — https://github.com/chippers/serialize-to-javascript
serialize-to-javascript-impl 0.1.2 — MIT OR Apache-2.0 — https://github.com/chippers/serialize-to-javascript
serialport 4.8.1 — MPL-2.0 — https://github.com/serialport/serialport-rs
servo_arc 0.2.0 — MIT OR Apache-2.0 — https://github.com/servo/servo
sha1 0.10.6 — MIT OR Apache-2.0 — https://github.com/RustCrypto/hashes
sha2 0.10.9 — MIT OR Apache-2.0 — https://github.com/RustCrypto/hashes
shlex 1.3.0 — MIT OR Apache-2.0 — https://github.com/comex/rust-shlex
simd-adler32 0.3.8 — MIT — https://github.com/mcountryman/simd-adler32
siphasher 0.3.11 — MIT/Apache-2.0 — https://github.com/jedisct1/rust-siphash
siphasher 1.0.1 — MIT/Apache-2.0 — https://github.com/jedisct1/rust-siphash
slab 0.4.11 — MIT — https://github.com/tokio-rs/slab
smallvec 1.15.1 — MIT OR Apache-2.0 — https://github.com/servo/rust-smallvec
socket2 0.6.1 — MIT OR Apache-2.0 — https://github.com/rust-lang/socket2
softbuffer 0.4.8 — MIT OR Apache-2.0 — https://github.com/rust-windowing/softbuffer
soup3 0.5.0 — MIT — https://gitlab.gnome.org/World/Rust/soup3-rs
soup3-sys 0.5.0 — MIT — https://gitlab.gnome.org/World/Rust/soup3-rs
stable_deref_trait 1.2.1 — MIT OR Apache-2.0 — https://github.com/storyyeller/stable_deref_trait
string_cache 0.8.9 — MIT OR Apache-2.0 — https://github.com/servo/string-cache
string_cache_codegen 0.5.4 — MIT OR Apache-2.0 — https://github.com/servo/string-cache
strsim 0.11.1 — MIT — https://github.com/rapidfuzz/strsim-rs
subtle 2.6.1 — BSD-3-Clause — https://github.com/dalek-cryptography/subtle
swift-rs 1.0.7 — MIT OR Apache-2.0 — https://github.com/Brendonovich/swift-rs
syn 1.0.109 — MIT OR Apache-2.0 — https://github.com/dtolnay/syn
syn 2.0.114 — MIT OR Apache-2.0 — https://github.com/dtolnay/syn
sync_wrapper 1.0.2 — Apache-2.0 — https://github.com/Actyx/sync_wrapper
synstructure 0.13.2 — MIT — https://github.com/mystor/synstructure
sysinfo 0.30.13 — MIT — https://github.com/GuillaumeGomez/sysinfo
system-deps 6.2.2 — MIT OR Apache-2.0 — https://github.com/gdesmott/system-deps
tao 0.34.5 — Apache-2.0 — https://github.com/tauri-apps/tao
tao-macros 0.1.3 — MIT OR Apache-2.0 — https://github.com/tauri-apps/tao
tar 0.4.44 — MIT OR Apache-2.0 — https://github.com/alexcrichton/tar-rs
target-lexicon 0.12.16 — Apache-2.0 WITH LLVM-exception — https://github.com/bytecodealliance/target-lexicon
tauri 2.9.5 — Apache-2.0 OR MIT — https://github.com/tauri-apps/tauri
tauri-build 2.5.3 — Apache-2.0 OR MIT — https://github.com/tauri-apps/tauri
tauri-codegen 2.5.2 — Apache-2.0 OR MIT — https://github.com/tauri-apps/tauri
tauri-macros 2.5.2 — Apache-2.0 OR MIT — https://github.com/tauri-apps/tauri
tauri-plugin 2.5.2 — Apache-2.0 OR MIT — https://github.com/tauri-apps/tauri
tauri-plugin-updater 2.9.0 — Apache-2.0 OR MIT — https://github.com/tauri-apps/plugins-workspace
tauri-runtime 2.9.2 — Apache-2.0 OR MIT — https://github.com/tauri-apps/tauri
tauri-runtime-wry 2.9.3 — Apache-2.0 OR MIT — https://github.com/tauri-apps/tauri
tauri-utils 2.8.1 — Apache-2.0 OR MIT — https://github.com/tauri-apps/tauri
tauri-winres 0.3.5 — MIT — https://github.com/tauri-apps/winres
tempfile 3.24.0 — MIT OR Apache-2.0 — https://github.com/Stebalien/tempfile
tendril 0.4.3 — MIT/Apache-2.0 — https://github.com/servo/tendril
thiserror 1.0.69 — MIT OR Apache-2.0 — https://github.com/dtolnay/thiserror
thiserror 2.0.17 — MIT OR Apache-2.0 — https://github.com/dtolnay/thiserror
thiserror-impl 1.0.69 — MIT OR Apache-2.0 — https://github.com/dtolnay/thiserror
thiserror-impl 2.0.17 — MIT OR Apache-2.0 — https://github.com/dtolnay/thiserror
time 0.3.45 — MIT OR Apache-2.0 — https://github.com/time-rs/time
time-core 0.1.7 — MIT OR Apache-2.0 — https://github.com/time-rs/time
time-macros 0.2.25 — MIT OR Apache-2.0 — https://github.com/time-rs/time
tiny_http 0.12.0 — MIT OR Apache-2.0 — https://github.com/tiny-http/tiny-http
tinystr 0.8.2 — Unicode-3.0 — https://github.com/unicode-org/icu4x
tinyvec 1.10.0 — Zlib OR Apache-2.0 OR MIT — https://github.com/Lokathor/tinyvec
tinyvec_macros 0.1.1 — MIT OR Apache-2.0 OR Zlib — https://github.com/Soveu/tinyvec_macros
tokio 1.49.0 — MIT — https://github.com/tokio-rs/tokio
tokio-rustls 0.26.4 — MIT OR Apache-2.0 — https://github.com/rustls/tokio-rustls
tokio-util 0.7.18 — MIT — https://github.com/tokio-rs/tokio
toml 0.8.2 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
toml 0.9.11+spec-1.1.0 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
toml_datetime 0.6.3 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
toml_datetime 0.7.5+spec-1.1.0 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
toml_edit 0.19.15 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
toml_edit 0.20.2 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
toml_edit 0.23.10+spec-1.0.0 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
toml_parser 1.0.6+spec-1.1.0 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
toml_writer 1.0.6+spec-1.1.0 — MIT OR Apache-2.0 — https://github.com/toml-rs/toml
tower 0.5.3 — MIT — https://github.com/tower-rs/tower
tower-http 0.6.8 — MIT — https://github.com/tower-rs/tower-http
tower-layer 0.3.3 — MIT — https://github.com/tower-rs/tower
tower-service 0.3.3 — MIT — https://github.com/tower-rs/tower
tracing 0.1.44 — MIT — https://github.com/tokio-rs/tracing
tracing-core 0.1.36 — MIT — https://github.com/tokio-rs/tracing
tray-icon 0.21.3 — MIT OR Apache-2.0 — https://github.com/tauri-apps/tray-icon
try-lock 0.2.5 — MIT — https://github.com/seanmonstar/try-lock
tungstenite 0.21.0 — MIT OR Apache-2.0 — https://github.com/snapview/tungstenite-rs
typeid 1.0.3 — MIT OR Apache-2.0 — https://github.com/dtolnay/typeid
typenum 1.19.0 — MIT OR Apache-2.0 — https://github.com/paholg/typenum
unescaper 0.1.8 — GPL-3.0/MIT — https://github.com/hack-ink/unescaper
unic-char-property 0.9.0 — MIT/Apache-2.0 — https://github.com/open-i18n/rust-unic/
unic-char-range 0.9.0 — MIT/Apache-2.0 — https://github.com/open-i18n/rust-unic/
unic-common 0.9.0 — MIT/Apache-2.0 — https://github.com/open-i18n/rust-unic/
unic-ucd-ident 0.9.0 — MIT/Apache-2.0 — https://github.com/open-i18n/rust-unic/
unic-ucd-version 0.9.0 — MIT/Apache-2.0 — https://github.com/open-i18n/rust-unic/
unicode-ident 1.0.22 — (MIT OR Apache-2.0) AND Unicode-3.0 — https://github.com/dtolnay/unicode-ident
unicode-segmentation 1.12.0 — MIT OR Apache-2.0 — https://github.com/unicode-rs/unicode-segmentation
untrusted 0.9.0 — ISC — https://github.com/briansmith/untrusted
url 2.5.8 — MIT OR Apache-2.0 — https://github.com/servo/rust-url
urlpattern 0.3.0 — MIT — https://github.com/denoland/rust-urlpattern
utf-8 0.7.6 — MIT OR Apache-2.0 — https://github.com/SimonSapin/rust-utf8
utf8_iter 1.0.4 — Apache-2.0 OR MIT — https://github.com/hsivonen/utf8_iter
uuid 1.19.0 — Apache-2.0 OR MIT — https://github.com/uuid-rs/uuid
vcpkg 0.2.15 — MIT/Apache-2.0 — https://github.com/mcgoo/vcpkg-rs
version-compare 0.2.1 — MIT — https://gitlab.com/timvisee/version-compare
version_check 0.9.5 — MIT/Apache-2.0 — https://github.com/SergioBenitez/version_check
vswhom 0.1.0 — MIT — https://github.com/nabijaczleweli/vswhom.rs
vswhom-sys 0.1.3 — MIT — https://github.com/nabijaczleweli/vswhom-sys.rs
walkdir 2.5.0 — Unlicense/MIT — https://github.com/BurntSushi/walkdir
want 0.3.1 — MIT — https://github.com/seanmonstar/want
wasi 0.11.1+wasi-snapshot-preview1 — Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT — https://github.com/bytecodealliance/wasi
wasi 0.9.0+wasi-snapshot-preview1 — Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT — https://github.com/bytecodealliance/wasi
wasip2 1.0.2+wasi-0.2.9 — Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT — https://github.com/bytecodealliance/wasi-rs
wasm-bindgen 0.2.108 — MIT OR Apache-2.0 — https://github.com/wasm-bindgen/wasm-bindgen
wasm-bindgen-futures 0.4.58 — MIT OR Apache-2.0 — https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures
wasm-bindgen-macro 0.2.108 — MIT OR Apache-2.0 — https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro
wasm-bindgen-macro-support 0.2.108 — MIT OR Apache-2.0 — https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro-support
wasm-bindgen-shared 0.2.108 — MIT OR Apache-2.0 — https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared
wasm-streams 0.4.2 — MIT OR Apache-2.0 — https://github.com/MattiasBuelens/wasm-streams/
web-sys 0.3.85 — MIT OR Apache-2.0 — https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys
web-time 1.1.0 — MIT OR Apache-2.0 — https://github.com/daxpedda/web-time
webkit2gtk 2.0.1 — MIT — https://github.com/tauri-apps/webkit2gtk-rs
webkit2gtk-sys 2.0.1 — MIT — https://github.com/tauri-apps/webkit2gtk-rs
webpki-roots 1.0.5 — CDLA-Permissive-2.0 — https://github.com/rustls/webpki-roots
webview2-com 0.38.2 — MIT — https://github.com/wravery/webview2-rs
webview2-com-macros 0.8.1 — MIT — https://github.com/wravery/webview2-rs
webview2-com-sys 0.38.2 — MIT — https://github.com/wravery/webview2-rs
winapi 0.3.9 — MIT/Apache-2.0 — https://github.com/retep998/winapi-rs
winapi-i686-pc-windows-gnu 0.4.0 — MIT/Apache-2.0 — https://github.com/retep998/winapi-rs
winapi-util 0.1.11 — Unlicense OR MIT — https://github.com/BurntSushi/winapi-util
winapi-x86_64-pc-windows-gnu 0.4.0 — MIT/Apache-2.0 — https://github.com/retep998/winapi-rs
window-vibrancy 0.6.0 — Apache-2.0 OR MIT — https://github.com/tauri-apps/tauri-plugin-vibrancy
windows 0.52.0 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows 0.61.3 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-collections 0.2.0 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-core 0.52.0 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-core 0.61.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-core 0.62.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-future 0.2.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-implement 0.60.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-interface 0.59.3 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-link 0.1.3 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-link 0.2.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-numerics 0.2.0 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-result 0.3.4 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-result 0.4.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-strings 0.4.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-strings 0.5.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-sys 0.45.0 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-sys 0.52.0 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-sys 0.59.0 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-sys 0.60.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-sys 0.61.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-targets 0.42.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-targets 0.52.6 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-targets 0.53.5 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-threading 0.1.0 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows-version 0.1.7 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_aarch64_gnullvm 0.42.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_aarch64_gnullvm 0.52.6 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_aarch64_gnullvm 0.53.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_aarch64_msvc 0.42.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_aarch64_msvc 0.52.6 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_aarch64_msvc 0.53.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_i686_gnu 0.42.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_i686_gnu 0.52.6 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_i686_gnu 0.53.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_i686_gnullvm 0.52.6 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_i686_gnullvm 0.53.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_i686_msvc 0.42.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_i686_msvc 0.52.6 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_i686_msvc 0.53.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_x86_64_gnu 0.42.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_x86_64_gnu 0.52.6 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_x86_64_gnu 0.53.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_x86_64_gnullvm 0.42.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_x86_64_gnullvm 0.52.6 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_x86_64_gnullvm 0.53.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_x86_64_msvc 0.42.2 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_x86_64_msvc 0.52.6 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
windows_x86_64_msvc 0.53.1 — MIT OR Apache-2.0 — https://github.com/microsoft/windows-rs
winnow 0.5.40 — MIT — https://github.com/winnow-rs/winnow
winnow 0.7.14 — MIT — https://github.com/winnow-rs/winnow
winreg 0.55.0 — MIT — https://github.com/gentoo90/winreg-rs
wit-bindgen 0.51.0 — Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT — https://github.com/bytecodealliance/wit-bindgen
writeable 0.6.2 — Unicode-3.0 — https://github.com/unicode-org/icu4x
wry 0.53.5 — Apache-2.0 OR MIT — https://github.com/tauri-apps/wry
x11 2.21.0 — MIT — https://github.com/AltF02/x11-rs.git
x11-dl 2.21.0 — MIT — https://github.com/AltF02/x11-rs.git
xattr 1.6.1 — MIT OR Apache-2.0 — https://github.com/Stebalien/xattr
yoke 0.8.1 — Unicode-3.0 — https://github.com/unicode-org/icu4x
yoke-derive 0.8.1 — Unicode-3.0 — https://github.com/unicode-org/icu4x
zerocopy 0.8.33 — BSD-2-Clause OR Apache-2.0 OR MIT — https://github.com/google/zerocopy
zerocopy-derive 0.8.33 — BSD-2-Clause OR Apache-2.0 OR MIT — https://github.com/google/zerocopy
zerofrom 0.1.6 — Unicode-3.0 — https://github.com/unicode-org/icu4x
zerofrom-derive 0.1.6 — Unicode-3.0 — https://github.com/unicode-org/icu4x
zeroize 1.8.2 — Apache-2.0 OR MIT — https://github.com/RustCrypto/utils
zerotrie 0.2.3 — Unicode-3.0 — https://github.com/unicode-org/icu4x
zerovec 0.11.5 — Unicode-3.0 — https://github.com/unicode-org/icu4x
zerovec-derive 0.11.2 — Unicode-3.0 — https://github.com/unicode-org/icu4x
zip 4.6.1 — MIT — https://github.com/zip-rs/zip2.git
zmij 1.0.14 — MIT — https://github.com/dtolnay/zmij
```

</details>
