
if ($IsWindows) {
    Write-Host enable Windows developer mode!
}

git config --global core.symlinks true
git config --system core.longpaths true

git submodule update --init --recursive
