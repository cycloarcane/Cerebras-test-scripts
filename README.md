# Cerebras Test Scripts

```
 .---. .----..----. .----..----. .----.   .--.   .----.
/  ___}| {_  | {}  }| {_  | {}  }| {}  } / {} \ { {__  
\     }| {__ | .-. \| {__ | {}  }| .-. \/  /\  \.-._} }
 `---' `----'`-' `-'`----'`----' `-' `-'`-'  `-'`----' 
 .---. .----. .----..---. .-..-. .-. .---.             
{_   _}| {_  { {__ {_   _}| ||  `| |/   __}            
  | |  | {__ .-._} } | |  | || |\  |\  {_ }            
  `-'  `----'`----'  `-'  `-'`-' `-' `---'             
```

Welcome to the **Cerebras Test Scripts** repository! This project contains a collection of scripts designed to test and demonstrate the superfast inference capabilities of the [Cerebras AI API](https://www.cerebras.net/). The goal is to showcase how these scripts can be utilized in open-source projects and applications to leverage the power of Cerebras AI.

## Getting Started

### Prerequisites

- Python 3.8+ or Rust (if using Rust scripts)
- Access to the Cerebras AI API (You can sign up [here](https://inference.cerebras.ai/))

### Installation

Clone this repository to your local machine:

```bash
git clone https://github.com/cycloarcane/Cerebras-test-scripts.git
```

Navigate to the project directory:

```bash
cd Cerebras-test-scripts
```

### For Python Scripts

Install the required dependencies:

```bash
pip install -r requirements.txt
```

### For Rust Scripts

If you are working with Rust files, ensure that you have Rust installed. You can install it via [rustup](https://www.rust-lang.org/tools/install):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Once Rust is installed, navigate to the directory containing the Rust file:

```bash
cd rust_project_directory
```

To build the Rust project, run:

```bash
cargo build
```

To execute the Rust project, use:

```bash
cargo run
```

### Usage

For Python scripts, run a script with:

```bash
python <script_name>.py
```

For Rust scripts, use `cargo run` as described above. Replace `<script_name>` or run the Rust project accordingly.

## Contributing

Contributions are welcome! If you have suggestions for improvements or new scripts, feel free to open an issue or submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

For any questions or inquiries, please reach out via email:

📧 cycloarkane@gmail.com

GitHub: [cycloarcane](https://github.com/cycloarcane)