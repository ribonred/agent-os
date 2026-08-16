# Build environment for the golden image.
#
# Pinned to the same release the image targets. That is the whole point:
# debootstrap needs a script for the target suite, and a host older than
# the target does not have one. Building inside the target release also
# keeps grub-install, mkfs and the e2fsprogs feature set in step with
# what the image will actually boot with.
#
# This container BUILDS a filesystem tree. It never runs the desktop, the
# services or the agent -- those are only unpacked into a directory here,
# with policy-rc.d blocking every package's attempt to start anything.
# A container cannot tell you the image boots; only hardware can.
#
#   docker build -t agentic-os-build build/
#   make rootfs-docker
FROM ubuntu:26.04

# Everything the three stages shell out to. Kept explicit rather than
# leaning on the base image, so a slimmer base upstream cannot silently
# remove a tool a later stage depends on.
RUN apt-get update && apt-get install -y --no-install-recommends \
        debootstrap \
        ca-certificates \
        curl \
        # git and ssh are used by the agent-runtime install. They exist
        # inside the built tree too, but the build itself reaches for
        # them before that tree is usable.
        git \
        openssh-client \
        # stage 2: partition table, filesystems, image assembly
        gdisk \
        parted \
        dosfstools \
        e2fsprogs \
        rsync \
        # tar --zstd shells out to this binary, and the image is
        # compressed with it
        zstd \
        xz-utils \
        # loop device handling and mountpoint checks
        util-linux \
        mount \
        # grub-install runs chrooted into the target tree, but the
        # packages come from the target's own apt -- these are for the
        # host side of that operation
        grub2-common \
        && rm -rf /var/lib/apt/lists/*

# The scripts detect their own repo root, so no WORKDIR assumption is
# baked in beyond the mount point make uses.
WORKDIR /repo

# Nothing runs by default: which stage to run is the caller's choice.
CMD ["/bin/bash"]
