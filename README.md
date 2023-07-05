# Lambda AOT Compiler (Morehead Lambda Compiler)

The Lambda programming language's implementation of an ahead-of-time (AOT)
    compiler can be found in this repository. This work, carried out as a URF
    (Undergraduate Research Fellowship) student researcher at Morehead State
    University, serves as Dalton Hensley's Bachelor's thesis.


## Introduction

With the help of the Lambda AOT Compiler, Lambda programs can be effectively
converted into machine code that can be executed directly by the target
hardware. This project's main goal is to investigate how utilizing AOT
compilation techniques might enhance the performance of Lambda programs.

## About the Author

Dalton Hensley is a student at Morehead State University pursuing a Bachelor's
degree in Computer Science. As an enthusiastic researcher, Dalton's academic
journey has been focused on exploring various aspects of programming languages
and compiler design. This project represents a significant milestone in
Dalton's undergraduate research experience.

## Compiler Design

The Lambda AOT Compiler adheres to a common compilation pipeline that includes
a series of phases, each helping to transform the source code into
optimized machine code. The following are the key phases of the compiler's design:

1. **Preprocessing**: In the preprocessing stage, the compiler performs initial
   transformations on the source code. This may involve removing comments and
   handling preprocessor directives.

2. **Lexical Analysis**: The next step is lexical analysis, which involves
   tokenizing the source code. In order to identify and classify tokens like
   keywords, identifiers, operators, and literals, the compiler scans the input
   program. This stage is essential for giving the succeeding stages a
   structured representation of the source code.

3. **Parsing**: Building an AST or a parse tree involves the process of
   parsing, which involves analyzing the token sequence produced by the lexical
   analysis stage. The parser checks the code's syntactic correctness using a
   given grammar to make sure it follows the syntax conventions of the
   language.

4. **Semantic Analysis**: Semantic analysis is carried out by the compiler
   after parsing the code. This phase verifies the program's meaning and
   validity. It involves operations such as type checking, scope resolution,
   and the detection of semantic mistakes. The compiler clarifies any
   ambiguities or inconsistencies and makes sure the program complies with the
   semantics of the language.

5. **LLVM IR Code Generation**: The compiler generates intermediate code in the
   LLVM (Low-Level Virtual Machine) representation after the code has undergone
   semantic analysis. For a variety of target architectures, LLVM offers a set
   of reusable compiler infrastructure components that simplify optimization
   and code generation. Before further optimization and translation to machine
   code, the generated LLVM code serves as an intermediary representation.

6. **Optimization**: The compiler uses a number of optimization techniques to
   boost the effectiveness and performance of the code it generates. These
   improvements include loop optimization, dead code removal, constant folding,
   and more. The goal of optimization is to speed up program execution, use
   less memory, and improve overall program quality.

7. **Machine Code Generation**: The optimized LLVM intermediate code is
   converted into machine code tailored to the target hardware architecture
   during the pipeline's final stage of compilation. The code is converted into
   executable instructions in this step, producing an AOT-compiled binary that
   can be directly executed by the target hardware.

By following this compilation pipeline, the Lambda AOT Compiler ensures the
accurate translation of Lambda source code into efficient and optimized machine
code.

## LLVM Intermediate Representation (IR)
When code is being compiled, LLVM's flexible and effective intermediate
representation (IR) acts as a representation that is independent of the
hardware being used. The semantics of a program are captured by the low-level,
strongly typed, and well-defined language known as LLVM IR.

Between the frontend (parsing and semantic analysis) and the backend (code
generation), the Lambda AOT Compiler utilizes LLVM IR. The compiler can access
a variety of potent optimization techniques and can target various hardware
architectures without the need for manual reimplementation by converting Lambda
source code into LLVM IR.

The typed instructions that make up the LLVM IR are used to represent different
operations, control flow diagrams, and memory access. In addition to supporting
high-level constructs like functions, loops, conditionals, and intricate data
structures, it offers a wide range of optimization features.

## LLVM Compiler Backend
The task of converting LLVM IR into machine code that is optimized for the
target hardware architecture is performed by the LLVM compiler backend. It is
made up of a number of elements and optimizations that convert the LLVM IR into
effective and quick machine code.


## Repository Structure

- `src/`: Contains the source code of the Lambda AOT Compiler.
- `shared/`: Contains the source code of the Lambda AOT Compiler libraries.
- `lynx/`: Contains Lambda's automated build system. 
- `examples/`: Includes example Lambda programs to test the compiler.
- `docs/`: Documentation and related resources.
- `tests/`: Test cases to validate the compiler's functionality.

## Getting Started

To get started with the Lambda AOT Compiler, follow these steps:

1. Clone the repository:

   ```shell
   git clone https://github.com/your-

username/repository-name.git
   ```

2. Navigate to the project directory:

   ```shell
   cd repository-name
   ```

3. Build and compile the Lambda AOT Compiler:

   ```shell
   # Instructions for building the compiler
   ```

4. Run the compiler on a Lambda source file:

   ```shell
   # Instructions for running the compiler
   ```

For more detailed instructions and usage examples, please refer to the
documentation in the `docs/` directory.

## Installing Rust on Windows

    1. Visit the official Rust website at https://www.rust-lang.org/.

    2. Click on the "Install" button located on the top right corner of the
       website.

    3. Scroll down to the section titled "Other Installers" and click on the
       "Windows" tab.

    4. You will see a link labeled "rustup‑init.exe." Click on it to download
       the Rust installation executable.

    5. Once the download is complete, run the rustup‑init.exe file.

    6. Follow the on-screen instructions to proceed with the installation. By
        default, the installer will choose the recommended options, so you can
        simply press Enter to proceed.

    7. During the installation, you will be prompted to review the license
       terms. Press Enter to accept the terms.

    8. The installer will then download and install the necessary components,
       which may take a few minutes.

    9. Once the installation is complete, you will see a message indicating
       that Rust has been installed successfully.

    10. Open a new command prompt or PowerShell window to verify the
        installation. Type rustc --version and press Enter. You should see the
        installed Rust version printed on the screen.


## Contributions

Contributions to the Lambda AOT Compiler project are welcome. If you find any
bugs, have suggestions for improvements, or would like to add new features,
please submit an issue or a pull request.

Please ensure that you adhere to the established coding conventions and follow
the project's code of conduct.

## License

The Lambda AOT Compiler is released under the [MIT
License](https://opensource.org/licenses/MIT). See the [LICENSE](LICENSE) file
for more information.

## Acknowledgements

We would like to express our gratitude to the following individuals for their
contributions and support:

- Dr. Heba Elgazzar: Research advisor and supervisor
- Morehead State University: Providing the resources and support necessary to
  conduct this research
- Open-source community: For the tools, libraries, and frameworks that made
  this project possible

## Contact

For any inquiries or further information, please contact Dalton Hensley at
dzhensley@moreheadstate.edu
