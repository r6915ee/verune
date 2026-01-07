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

# Bump the project version. Requires git-cliff and cargo-edit. First two parameters are passed to git-cliff as bump version types.
bump main="" lib="" set-manifest-version="1" use-git="1" git-tag="1":
    #!/usr/bin/env sh
    bumped_vers=("{{ main }}" "{{ lib }}")
    include=("0" "1")
    count=0
    main_ver=""
    for i in "${bumped_vers[@]}"
    do
        if [[ ${include[${count}]} == "0" ]]; then
            ver=$(git cliff --bump $i --exclude-path "./libver/" --bumped-version)
            git cliff --bump $i --unreleased --exclude-path "./libver/" -o ./CHANGELOG.md
            if [[ {{ set-manifest-version }} == "1" ]]
            then
                main_ver=$(echo ${ver} | sed "s/v//g")
                cargo set-version ${main_ver}
            fi
        else
            ver=$(git cliff --bump $i --include-path "./libver/" --bumped-version)
            git cliff --bump $i --unreleased --include-path "./libver/" -o ./libver/CHANGELOG.md
            if [[ {{ set-manifest-version }} == "1" ]]
            then
                cargo set-version -p libver $(echo ${ver} | sed "s/v//g")
            fi
        fi
        (( count++ ))
    done
    if [[ {{ set-manifest-version }} == "1" ]] && [[ {{ use-git }} == "1" ]]
    then
        git stash push . ":!*CHANGELOG.md" ":!*Cargo*"
        git add .
        git commit -m "chore(release): prepare for v${main_ver}"
        git stash pop
        if [[ {{ git-tag }} == "1" ]]; then
            git tag "v${main_ver}"
        fi
    fi
