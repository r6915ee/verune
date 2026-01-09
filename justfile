# Triggers dev recipe.
default: dev

# Build a specified profile.
build profile="dev":
    cargo build --profile {{ profile }}

# Build the "dev" profile.
dev: (build "dev")

# Build the "release" profile.
release: (build "release")

# Run the program.
run:
    cargo run

# Trigger all tests.
test:
    cargo test

# Trigger all tests, except with no default features enabled.
test-no-default-features:
    cargo test --no-default-features

# Trigger all tests, plus allow access to stdout.
test-out:
    cargo test -- --nocapture

# Trigger Clippy.
lint:
    cargo clippy

# Trigger rustfmt.
fmt:
    cargo fmt --all

# Immediately generate the documentation without testing.
doc-im:
    cargo doc --no-deps --workspace

# Test, and then generate the documentation.
doc: doc-im
    cargo test --doc
    cargo doc --no-deps --workspace

# Trigger doc recipe, and open  in target/doc.
view-docs: doc
    ${HTTP_SERVER} target/doc

# Bump the project version. Requires git-cliff, cargo-edit and sed. Use "none" to disable version bumping on libver.
bump main="auto" lib="auto" set-manifest-version="a" use-git="a" git-tag="a":
    #!/usr/bin/env sh
    bumped_vers=("{{ main }}" "{{ lib }}")
    include=("0" "1")
    count=0
    main_ver=""
    for i in "${bumped_vers[@]}"
    do
        if [ "${i}" = "major" ]; then
            i="minor"
            echo "Both verune and libver follow 0ver version scheme and cannot use major version numbers"
        fi
        if [ "${include[${count}]}" = "0" ]; then
            if [ "${i}" = "none" ]; then
                i="auto"
                echo "Only libver can be bumped using none parameter"
            fi
            ver=$(git cliff --bump $i --exclude-path "libver/" --bumped-version)
            echo "verune: ${ver}"
            git cliff --bump $i --unreleased --exclude-path "libver/" -o
            if [ -n "{{ set-manifest-version }}" ]
            then
                main_ver=$(echo ${ver} | sed "s/v//g")
                # set-version searches through all crates without a package ID, so we need to explicitly specify it.
                cargo set-version -p verune ${main_ver}
            fi
        else
            if [ "${i}" != "none" ]; then
                ver=$(git cliff --bump $i --include-path "libver/" --bumped-version)
                echo "libver: ${ver}"
                git cliff --bump $i --unreleased --include-path "libver/" -o libver/CHANGELOG.md
                if [ -n "{{ set-manifest-version }}" ]
                then
                    lib_ver=$(echo ${ver} | sed "s/v//g")
                    cargo set-version -p libver ${lib_ver}
                fi
            fi
        fi
        (( count++ ))
    done
    if [ -n "{{ set-manifest-version }}" ] && [ -n "{{ use-git }}" ]
    then
        git stash push . ":!*CHANGELOG.md" ":!*Cargo*"
        git add .
        git commit -m "chore(release): prepare for v${main_ver}"
        git stash pop
        if [ -n "{{ git-tag }}" ]; then
            git tag "v${main_ver}"
        fi
    fi
