#!/bin/sh

DIR_ENV=".env/"
if [ ! -d "$DIR_ENV" ]; then
    echo "Creating virtual enviroment ${DIR_ENV}..."
    python3 -m venv .env
fi

echo "Sourcing virtual enviroment.."
source ".env/bin/activate"

echo "Installing requirements.."
pip install -r requirements.txt

echo "Building and installing check_bp.."
cd check_bp && ./setup.sh
cd ..

echo "Building and installing rust_python_kyber.."
cd rust_python_kyber && ./setup.sh
cd ..
