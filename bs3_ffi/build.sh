set -euxo pipefail

echo "building node"
npm i
npm run build

echo "building native module"
nj-cli build
