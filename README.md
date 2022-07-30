# Pdf_Split
A lightweight app to split pdf into separate files. Easy to use as it comes with a graphical user interface.

## Installation

**1. [Install](https://www.rust-lang.org/tools/install) the Rust programming language**
**2. Clone this repository**<br>
**3. Build by running the below command**<br>
&nbsp;&nbsp;&nbsp;&nbsp; `cargo build --release`

## Usage

Find the binary file in the `../pdf/target/release` directory
On a linux distribution, make it executable on windows and mac it should work out of the box.

Run the application. 

In the graphical user interface follow these steps:

1. Choose a file or type the absolute path in.
2. Define the page range you want to extract and give the new file a name (include *.pdf*)
3. if you want to extract more than one range, click the "Add" button and repeat step 2.
4. Click "Do Split".

The new files will be in the same directory as the original file.


