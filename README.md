# Install

```
sudo apt-get install libcgns-dev
cargo install --git https://github.com/tucanos/mhoverlap.git mhoverlap-cli
```

# Usage

```
Tag elements of a CGNS file which overlap with an other CGNS file

Usage: mhoverlap-cli [OPTIONS] <REFERENCE> <CHECK> <RESULT>

Arguments:
  <REFERENCE>  Reference CGNS file
  <CHECK>      CGNS file to check
  <RESULT>     Result CGNS file

Options:
  -t, --tolerance <TOLERANCE>  Tolerance value. Automatically computed if not specified
  -h, --help                   Print help
```
