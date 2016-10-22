# dlrs

A wget and curl replacement written in Rust.

This does not have any of the features of wget or curl yet.
In fact all it does is downloading from http or ftp to stdout or a local file.
However as it's written in Rust, it is super secure!

In future this program should check by what name it was called and interpret the arguments accordingly, so it can be used as a drop-in replacement for wget and curl.
There should also fly a Rust *library* out of this to facilitate downloading of arbitrary stuff in Rust.

## Usage

    Usage: dlrs url [outfilename]
    If no outfilename is given, will download to stdout

## Contributing

Please contribute to the design discussion in the Issues.
Please contribute further transports (scp would be nice).
Contribute all the stuff!
