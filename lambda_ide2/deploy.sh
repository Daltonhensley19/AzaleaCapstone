

cd ../ && CARGO_TARGET_DIR=./lambda_ide/compiler cargo build -r \
    && CARGO_TARGET_DIR=./lambda_ide/compiler cargo build
