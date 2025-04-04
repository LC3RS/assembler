<!-- Improved compatibility of back to top link: See: https://github.com/LC3RS/assembler/pull/73 -->
<a id="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![Unlicense License][license-shield]][license-url]
<!-- [![LinkedIn][linkedin-shield]][linkedin-url] -->

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/LC3RS/assembler">
    <img src="https://avatars.githubusercontent.com/u/201461929?s=500&v=4" alt="Logo" width="100">
  </a>

  <h3 align="center">Assembler</h3> 

  <p align="center">
    Assembler for LC-3 Virtual Machine, written in Rust 
    <br />
    <!-- <a href="https://github.com/LC3RS/assembler/wiki"><strong>Explore the docs »</strong></a> -->
    <!-- <br /> -->
    <br />
    <!-- <a href="https://github.com/LC3RS/assembler">View Demo</a> -->
    <!-- &middot; -->
    <a href="https://github.com/LC3RS/assembler/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/LC3RS/assembler/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
        <li><a href="#usage">Usage</a></li>
        <li><a href="#tests">Tests</a></li>
      </ul>
    </li>
    <li><a href="#top-contributors">Top Contributors</a></li>
  </ol>
</details>
 
---

<!-- ABOUT THE PROJECT -->
## About The Project

This project aims to implement an assembler for the LC-3 virtual machine.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Built With

[![Rust](https://img.shields.io/badge/Rust-%2300599C.svg?logo=rust&logoColor=white)](#)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

---

<!-- GETTING STARTED -->
## Getting Started

### Prerequisites


### Installation


```bash
git clone https://github.com/LC3RS/assembler.git
cd assembler
nix build

# cargo build --release
```


<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Usage

```bash
Usage: assembler [OPTIONS] --file <FILE>

Options:
  -f, --file <FILE>        Path to source assembly file
  -o, --outfile <OUTFILE>  Output file name (without extension) [default: out]
  -d, --debug              Turn on debug-mode
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version

```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Tests


```bash
cargo test
```


<p align="right">(<a href="#readme-top">back to top</a>)</p>

---

<!-- CONTRIBUTING -->
## Top contributors

<a href="https://github.com/LC3RS/assembler/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=LC3RS/assembler" alt="contrib.rocks image" />
</a>

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/LC3RS/assembler.svg?style=for-the-badge
[contributors-url]: https://github.com/LC3RS/assembler/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/LC3RS/assembler.svg?style=for-the-badge
[forks-url]: https://github.com/LC3RS/assembler/network/members
[stars-shield]: https://img.shields.io/github/stars/LC3RS/assembler.svg?style=for-the-badge
[stars-url]: https://github.com/LC3RS/assembler/stargazers
[issues-shield]: https://img.shields.io/github/issues/LC3RS/assembler.svg?style=for-the-badge
[issues-url]: https://github.com/LC3RS/assembler/issues
[license-shield]: https://img.shields.io/github/license/LC3RS/assembler.svg?style=for-the-badge
[license-url]: https://github.com/LC3RS/assembler/blob/master/LICENSE.txt
