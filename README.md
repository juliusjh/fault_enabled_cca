# fault_enabled_cca

Code belonging to [Fault-enabled chosen-ciphertext attack on Kyber](https://eprint.iacr.org/2021/1222.pdf).

To run the attack:

1. Create a virtual enviroment in .env
	```
	$ python3 -m venv .env
	```
2. Install numpy and scipy
	```
	$ pip install -r requirements.txt
	```
3. Install Rust (see https://www.rust-lang.org/learn/get-started)
	```	
	$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	```	
4. Compile the Rust crates to python modules using maturin
	```
	$ VER=kyber512 source maturin_build.sh
	```	
5. Execute the python script
	```
	$ ./python/main.py --threads [number of threads] --number [number of faults] --file [file/dir to save to] --runs [number of runs] --generate --save-keys --iterations [number of iterations] --zip-file [zip to]
	```
E.g. 

    $ VER=kyber512 source maturin_build.sh
    $ ./python/main.py --seed 42 --number 6000 --file ineqs/ineqs512 --runs 1 --generate --save-keys --iterations 10 --zip-file test.zip

Simulates the attack against Kyber512 with 6000 faults/inequalities, 1 run, generating new keys and inequalities, 10 belief propagation iterations, and saving to test.zip.

To also save the generated inequalities, use the --save option. Note that while inequalities are sampled using the seed provided (--seed), the keys are not sampled using the seed. Therefore, saving keys is recommended.
To reuse keys and inequalities, do not pass --generate.

In case of bugs or technical problems, please contact me (Julius) under the e-mail given in the paper.
	
