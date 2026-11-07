#!/usr/bin/env bash
# Run the existing cold-storage CLI with containerized Python, Git and Zstandard.
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)
root_dir=$repo_dir
usage() {
    printf 'Usage: bash scripts/linux/cold_storage_docker.sh [--root CHECKOUT] COMMAND [ARGS...]\n'
    printf 'Examples: batch; batch --apply; list; restore target/unit/resources\n'
    printf 'Requires local Linux Docker Engine and host PID visibility. No arguments show this help.\n'
}
if (( $# == 0 )) || [[ "$1" == --help || "$1" == -h ]]; then usage; exit 0; fi
if [[ "$1" == --root ]]; then
    if (( $# < 3 )); then usage >&2; exit 2; fi
    root_dir=$(cd -- "$2" && pwd -P)
    shift 2
fi
if [[ $(uname -s) != Linux || $(cat /proc/1/comm) != systemd ]]; then
    printf 'This launcher requires a Linux host with systemd and host PID visibility.\n' >&2; exit 1
fi
for path in "$root_dir" "$repo_dir"; do
    if [[ "$path" == *','* || "$path" == *'"'* || "$path" == *$'\n'* || "$path" == *$'\r'* ]]; then
        printf 'Docker mount paths must not contain commas, double quotes or newlines.\n' >&2; exit 2
    fi
done
command -v docker >/dev/null || { printf 'Docker Engine is required.\n' >&2; exit 1; }
if [[ -n ${DOCKER_HOST:-} && -z ${DOCKER_CONTEXT:-} ]]; then
    endpoint=$DOCKER_HOST
else
    endpoint=$(docker context inspect --format '{{.Endpoints.docker.Host}}' "$(docker context show)")
fi
if [[ "$endpoint" != unix://* ]]; then
    printf 'A local Unix-socket Docker daemon is required; remote hosts are unsupported.\n' >&2; exit 1
fi
daemon=$(docker info --format '{{.OSType}} {{.OperatingSystem}} {{json .SecurityOptions}}')
if [[ "$daemon" != linux\ * || "$daemon" == *rootless* || "$daemon" == *'Docker Desktop'* ]]; then
    printf 'Use local Linux Docker Engine; rootless Docker and Docker Desktop cannot provide this host bridge.\n' >&2; exit 1
fi
image=ffta-cold-storage:local
inputs=(Dockerfile.cold-storage Dockerfile.cold-storage.dockerignore
    scripts/cold_storage.py scripts/cold_storage_batch.py scripts/cold_storage_container.py
    scripts/artifact_coordinator.py scripts/local_artifacts.py)
fingerprint=$(cd -- "$repo_dir" && sha256sum -- "${inputs[@]}" | sha256sum)
fingerprint=${fingerprint%% *}
previous=$(docker image inspect --format '{{index .Config.Labels "org.ffta.cold-storage.source-sha256"}}' "$image" 2>/dev/null || true)
if [[ "$previous" != "$fingerprint" ]]; then
    docker build --file "$repo_dir/Dockerfile.cold-storage" --tag "$image" \
        --label "org.ffta.cold-storage.source-sha256=$fingerprint" "$repo_dir" >&2
fi
if [[ -L "$root_dir/target" ]]; then
    printf 'A symbolic-link target directory is not supported.\n' >&2; exit 2
fi
mkdir -p -- "$root_dir/target"
groups=()
read -r -a group_ids <<< "$(id -G)"
for group_id in "${group_ids[@]}"; do groups+=(--group-add "$group_id"); done
security=(--security-opt no-new-privileges)
# docker-default blocks reading host process details across AppArmor profiles.
# Only this container opts out; UID permissions, seccomp and dropped caps remain.
if [[ "$daemon" == *name=apparmor* ]]; then security+=(--security-opt apparmor=unconfined); fi
container_name="ffta-cold-storage-$$-$RANDOM"
exec docker run --rm --init --name "$container_name" --hostname "$container_name" \
    --pid host --userns host --user "$(id -u):$(id -g)" "${groups[@]}" \
    --cap-drop ALL "${security[@]}" --network none --read-only \
    --tmpfs /tmp:rw,nosuid,nodev,noexec,size=64m \
    --env "FFTA_STORAGE_DRIVER_PID=$$" \
    --env "FFTA_STORAGE_CONTAINER_NAME=$container_name" \
    --env "FFTA_STORAGE_PID_NAMESPACE=$(readlink /proc/self/ns/pid)" \
    --mount "type=bind,source=$root_dir,target=$root_dir,readonly" \
    --mount "type=bind,source=$root_dir/target,target=$root_dir/target" \
    --workdir "$root_dir" "$image" --root "$root_dir" "$@"
