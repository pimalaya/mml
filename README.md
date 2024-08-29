# 📫 MIME Meta Language [![GitHub release](https://img.shields.io/github/v/release/pimalaya/mml?color=success)](https://github.com/pimalaya/mml/releases/latest) [![Matrix](https://img.shields.io/matrix/pimalaya:matrix.org?color=success&label=chat)](https://matrix.to/#/#pimalaya:matrix.org)

CLI to compile [MML] messages to [MIME] messages and interpret [MIME] messages as [MML] messages, based on [`mml-lib`](https://github.com/pimalaya/core/tree/master/mml).

[MML]: https://www.gnu.org/software/emacs/manual/html_node/emacs-mime/MML-Definition.html
[MIME]: https://www.rfc-editor.org/rfc/rfc2045

## Features

- MML to MIME messages compilation (`mml compile --help`)
- MIME to MML messages interpretation (`mml interpret --help`)

## Installation

<table align="center">
<tr>
<td width="50%">
<a href="https://repology.org/project/mml/versions">
<img src="https://repology.org/badge/vertical-allrepos/mml.svg" alt="Packaging status" />
</a>
</td>
<td width="50%">

```bash
# Cargo
$ cargo install mml-cli

# Nix
$ nix-env -i mml
```

*See the [documentation](https://pimalaya.org/mml/cli/installation/index.html) for other installation methods.*

</td>
</tr>
</table>

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/project/Pimalaya/index.html)

Special thanks to the [NLnet foundation](https://nlnet.nl/project/Pimalaya/index.html) and the [European Commission](https://www.ngi.eu/) that helped the project to receive financial support from:

- [NGI Assure](https://nlnet.nl/assure/) in 2022
- [NGI Zero Entrust](https://nlnet.nl/entrust/) in 2023

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
