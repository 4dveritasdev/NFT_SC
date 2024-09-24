
    cd nft
    cargo partisia-contract build --release

    cd user
    cargo partisia-contract build --release

The compiled wasm/zkwa and abi files are located in

    target/wasm32-unknown-unknown/release

To run the test suite, run the following command:
    
    ./run-java-tests.sh

To generate the code coverage report, run the following command:
    
    cargo partisia-contract build --coverage

The coverage report will be located in java-test/target/coverage