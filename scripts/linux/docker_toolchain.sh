# Sourced by the Buildx launchers. Keep one local tag per toolchain recipe.
prepare_toolchain() {
    local platform=$1 recipe=$2 context=$3 driver digest image
    toolchain_args=()
    driver=$(docker buildx inspect) || return
    # A remote/container builder cannot use the engine's local image store.
    if [[ ! "$driver" =~ (^|$'\n')Driver:[[:blank:]]+docker($|$'\n') ]]; then return 0; fi
    digest=$(awk '/^# End of cached toolchain/{found=1; exit} {sub(/\r$/, ""); print} END {if (!found) exit 1}' "$recipe" | sha256sum) || return
    image="ffta-toolchain-${platform}:${digest:0:20}"
    if ! docker image inspect "$image" >/dev/null 2>&1; then
        docker buildx build --platform linux/amd64 --target toolchain \
            --file "$recipe" --output type=image,unpack=false --tag "$image" "$context" || return
    fi
    toolchain_args=(--build-arg "FFTA_TOOLCHAIN=$image")
    if [[ ${4:-} == export ]]; then
        local image_id
        image_id=$(docker image inspect --format '{{.Id}}' "$image") || return
        toolchain_args+=(--build-arg "FFTA_TOOLCHAIN_ID=$image_id")
    fi
}
