#!/bin/bash

# Since some cargo installs are done in the Dockerfile
# some ownerships are root which prevents trunk, e. g.,
# from installing dependencies.
sudo chown -R vscode /usr/local/cargo
