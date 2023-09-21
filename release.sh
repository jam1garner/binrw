#!/usr/bin/env bash

set -ueo pipefail

DEFAULT_REPO=jam1garner/binrw
DEFAULT_BRANCH=master

usage() {
	echo "Usage: $0 [branch] [version]"
	echo
	echo "Specifying a branch will create a new patch release from that branch."
	echo "Otherwise, a new release will be created from '$DEFAULT_BRANCH'."
	echo
	echo "Specifying a version will create a release with that specific version."
	echo "Otherwise, the version number for the release will be adapted from"
	echo "Cargo.toml."
	echo
	echo "After tagging, the version number in Cargo.toml will automatically be"
	echo "bumped unless a pre-release version number was passed explicitly to"
	echo "this script, in which case the version number in Cargo.toml will be"
	echo "restored."
	echo
	echo "If the version number for the new release ends in .0, a new minor"
	echo "release branch will also be created."
	echo
	echo "Supported environment variables:"
	echo "  REPO: The GitHub repository (user/repo) to use for the release."
	echo "        Defaults to $DEFAULT_REPO."
	exit 0
}

set_package_publish() {
	sed -i '' -e "s/^\\(publish[[:space:]]*=[[:space:]]*\\).*$/\\1$1/" Cargo.toml
}

set_package_version() {
	sed -i '' -e "s/^\\(version[[:space:]]*=[[:space:]]*\\)\"[^\"]*\"/\\1\"$1\"/" Cargo.toml
	sed -i '' -e "s/^\\(binrw_derive[[:space:]]*=.*version[[:space:]]*=[[:space:]]*\\)\"[^\"]*\"/\\1\"$1\"/" binrw/Cargo.toml
}

if [ "${1-}" == "--help" ]; then
	usage
elif [ -n "${1-}" ]; then
	BRANCH=$1
else
	BRANCH=$DEFAULT_BRANCH
fi

if [ -n "${2-}" ]; then
	VERSION=$2
	if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-.*)?$ ]]; then
		echo "Invalid version '$VERSION'; versions must be in the form"
		echo "\`major.minor.patch[-extra]\`."
		exit 1
	fi
else
	VERSION=
fi

REPO=${REPO:-$DEFAULT_REPO}
ROOT_DIR=$(cd "$(dirname "$0")" && pwd)
BUILD_DIR="$ROOT_DIR/build-release"

if [ -d "$BUILD_DIR" ]; then
	echo "Existing build directory detected at $BUILD_DIR"
	echo "Aborted."
	exit 1
fi

echo "This is an internal binrw release script!"
echo -n "Press 'y' to create a new binrw release from $REPO branch $BRANCH"
if [ -z "$VERSION" ]; then
	echo "."
else
	echo -e "\nwith version override $VERSION."
fi
echo "(You can abort pushing upstream later on if something goes wrong.)"
read -r -s -n 1

if [ "$REPLY" != "y" ]; then
	echo "Aborted."
	exit 0
fi

# Using a pristine copy of the repo for the release to avoid local changes
# making their way into the release process, and to avoid polluting the local
# repo with wrong changes if the release process is aborted
cd "$ROOT_DIR"
mkdir "$BUILD_DIR"
git clone --recursive "git@github.com:$REPO" "$BUILD_DIR"

cd "$BUILD_DIR"

# Newly created tags and updated branches are stored so they can be pushed all
# at once at the end after the other work is guaranteed to be successful
PUSH_BRANCHES="$BRANCH"

echo -e "\nBuilding $BRANCH branch...\n"

git checkout "$BRANCH"

# head is needed in case the version number was manually updated to something
# which has a tag with a number in it, otherwise there will be multiple matches
# on the line instead of just the first one
VERSION_CARGO_NO_TAGS=$(grep -o '^version[[:space:]]*=[[:space:]]*"[^"]*"' Cargo.toml | grep -o "[0-9][0-9.]*" | head -1)

if [ -z "$VERSION" ]; then
	VERSION=$VERSION_CARGO_NO_TAGS
	VERSION_NO_TAGS=$VERSION_CARGO_NO_TAGS
else
	VERSION_NO_TAGS=$(echo "$VERSION" | grep -o "[0-9][0-9.]*")
fi

# Converting the version number to an array to auto-generate the next
# release version number
IFS="." read -r -a PRE_VERSION_SPLIT <<< "$VERSION_NO_TAGS"

if [[ "$VERSION" =~ ^[0-9][0-9.]*\.0$ ]]; then
	# Minor release gets a branch
	MAKE_BRANCH="${PRE_VERSION_SPLIT[0]}.${PRE_VERSION_SPLIT[1]}"
	BRANCH_VERSION="${PRE_VERSION_SPLIT[0]}.${PRE_VERSION_SPLIT[1]}.$((PRE_VERSION_SPLIT[2] + 1))-pre"

	# The next release is usually going to be a minor release; if the next
	# version is to be a major release, the package version in Git will need
	# to be manually updated or someone will have to pass a major version number
	# to the release script at the last second
	PRE_VERSION="${PRE_VERSION_SPLIT[0]}.$((PRE_VERSION_SPLIT[1] + 1)).0-pre"
else
	# Patch releases do not get branches
	MAKE_BRANCH=
	BRANCH_VERSION=

	if [[ "$VERSION" == "$VERSION_NO_TAGS" ]]; then
		# The next release version will always be another patch version
		PRE_VERSION="${PRE_VERSION_SPLIT[0]}.${PRE_VERSION_SPLIT[1]}.$((PRE_VERSION_SPLIT[2] + 1))-pre"
	else
		# The version being released is a pre-release, so do not bump anything
		PRE_VERSION="$VERSION_CARGO_NO_TAGS-pre"
	fi
fi

TAG_VERSION="v$VERSION"

# At this point:
# $VERSION is the version of binrw that is being released
# $TAG_VERSION is the name that will be used for the Git tag for the release
# $PRE_VERSION is the next pre-release version of binrw that will be set on
# the original branch after tagging
# $MAKE_BRANCH is the name of the new minor release branch that should be
# created (if this is not a patch release)
# $BRANCH_VERSION is the pre-release version of binrw that will be set on the
# minor release branch

# Something is messed up and this release has already happened
if [ "$(git tag | grep -c "^$TAG_VERSION$")" -gt 0 ]; then
	echo -e "\nTag $TAG_VERSION already exists! Please check the branch.\n"
	exit 1
fi

set_package_version "$VERSION"
set_package_publish "true"

git commit -m "Updating metadata for $VERSION" -m "[ci skip]" Cargo.toml binrw/Cargo.toml
git tag -s -m "Release $VERSION" "$TAG_VERSION"

set_package_version "$PRE_VERSION"
set_package_publish "false # Use \`release.sh\`"

git commit -m "Updating source version to $PRE_VERSION" -m "[ci skip]" Cargo.toml binrw/Cargo.toml

if [ "$MAKE_BRANCH" != "" ]; then
	git checkout -b "$MAKE_BRANCH" "$TAG_VERSION"
	set_package_version "$BRANCH_VERSION"

	git commit -m "Updating source version to $BRANCH_VERSION" -m "[ci skip]" Cargo.toml binrw/Cargo.toml
	PUSH_BRANCHES="$PUSH_BRANCHES $MAKE_BRANCH"
	PUSH_BRANCHES_MSG=" and branches ${PUSH_BRANCHES}"
fi

echo -e "\nDone!\n"
echo "Please confirm packaging success, then press 'y', ENTER to push"
echo "tags $TAG_VERSION${PUSH_BRANCHES_MSG:-}, or any other key to bail."
read -r -p "> "

if [ "$REPLY" != "y" ]; then
	echo "Aborted."
	exit 0
fi

for BRANCH in $PUSH_BRANCHES; do
	git push origin "$BRANCH"
done

git push origin --tags
git checkout "$TAG_VERSION"
cargo publish -p binrw_derive
cargo publish -p binrw

cd "$ROOT_DIR"
rm -rf "$BUILD_DIR"

echo -e "\nAll done! Yay!"
