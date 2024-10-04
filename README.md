# [Dango](https://github.com/left-curve/left-curve/tree/main/dango) genesis CLI for spinup a local testnet

## How to install

1. Clone and compile the [**consensus**](https://github.com/cometbft/cometbft) layer:

    ```shell
    git clone https://github.com/cometbft/cometbft
    cd cometbft
    git checkout v0.38.10
    make install
    ```

2. Clone and install the [**execution**](https://github.com/left-curve/left-curve) layer:

    ```shell
    git clone https://github.com/left-curve/left-curve
    cd left-curve
    just install
    ```

3. Clone and install the [**genesis-cli**](https://github.com/Rhaki/dangod):

    ```shell
    git clone https://github.com/Rhaki/dangod
    cd dangod
    just install
    ```

## How to use

1. Generate the **dangod** config file:

    ```shell
    dangod g generate 2
    ```

    This command will create the **genesis.json** file in `$home/.dangod` with 2 accounts.
    Is possible to open this file to change the default settings.

2. Build the **app_state** in the consensus **genesis.json** file:

    ```shell
    dangod g build
    ```

3. Start the chain:

    ```shell
    dangod start
    ```

    This command start both **execution** and **application** layer

4. To cleanup the data and restart back from **1.**, run :

    ```shell
    dangod g reset
    ```
