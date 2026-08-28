# Behavioural check for the effects' run-context resolution.
#
# The property under test is the one that motivates the resolver's existence:
# a pull request whose reported branch is its TARGET branch must not select
# production. That situation is indistinguishable from a default-branch push
# by branch name alone, so the check drives the resolver against a stub
# identity-token endpoint (effect-run-context-stub-id-token-server.py) and
# asserts the selected mode for each combination of event kind and token
# presence.
#
# The stub rejects a wrong method, path, bearer credential, content type, or
# audience, so the token-present cases also pin the client side of the
# contract rather than only the decision logic.
#
# The fixture is a sibling file rather than a subdirectory because
# `cargoSourceFilter` in modules/rust.nix retains every directory regardless
# of content, so a new directory anywhere in the repository perturbs the
# content-addressed source of the workspace gates and forces them to rebuild.
{ self, ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      runContext = self.lib.mkEffectRunContext pkgs;
    in
    {
      checks.effect-run-context =
        pkgs.runCommand "effect-run-context-check"
          {
            nativeBuildInputs = [
              pkgs.coreutils
              pkgs.python3
              runContext
            ];
            stubServer = ./effect-run-context-stub-id-token-server.py;
            audience = self.lib.effectIdTokenAudience;
            meta.description = "check: herculesCI effects resolve production vs preview from the run context";
          }
          ''
            set +e -uo pipefail

            failures=0
            shortRev=abc1234

            expect() {
              if [ "$2" = "$3" ]; then
                echo "  ok   $1 = $3"
              else
                echo "  FAIL $1: expected '$2', got '$3'"
                failures=$((failures + 1))
              fi
            }

            resolve() {
              unset IRONSTAR_RUN_CONTEXT_SOURCE IRONSTAR_EVENT IRONSTAR_PR_NUMBER \
                IRONSTAR_IS_PRODUCTION IRONSTAR_BRANCH IRONSTAR_PREVIEW_SLUG
              local resolved
              resolved="$(ironstar-effect-run-context "$1" "$shortRev")" || return 1
              echo "$resolved" | sed 's/^/    /'
              eval "$resolved"
            }

            start_stub() {
              url_file="$(mktemp)"
              STUB_TASK_TOKEN=stub-task-token \
              STUB_AUDIENCE="$audience" \
              STUB_CLAIMS="$1" \
              STUB_URL_FILE="$url_file" \
                python3 "$stubServer" &
              stub_pid=$!
              for _ in $(seq 1 100); do
                if [ -s "$url_file" ]; then
                  break
                fi
                sleep 0.1
              done
              if [ ! -s "$url_file" ]; then
                echo "stub id-token server never reported a URL" >&2
                exit 1
              fi
              export NIXBOT_ID_TOKEN_REQUEST_URL="$(cat "$url_file")"
              export NIXBOT_ID_TOKEN_REQUEST_TOKEN=stub-task-token
            }

            stop_stub() {
              kill "$stub_pid" 2>/dev/null
              wait "$stub_pid" 2>/dev/null
              unset NIXBOT_ID_TOKEN_REQUEST_URL NIXBOT_ID_TOKEN_REQUEST_TOKEN
            }

            echo "case 1: no id token, push to the default branch"
            resolve main
            expect source branch-name "$IRONSTAR_RUN_CONTEXT_SOURCE"
            expect event push "$IRONSTAR_EVENT"
            expect production true "$IRONSTAR_IS_PRODUCTION"
            expect slug main "$IRONSTAR_PREVIEW_SLUG"

            echo "case 2: no id token, pull request as the current service reports it"
            resolve refs/pull/42/merge
            expect source branch-name "$IRONSTAR_RUN_CONTEXT_SOURCE"
            expect event pull_request "$IRONSTAR_EVENT"
            expect pr-number 42 "$IRONSTAR_PR_NUMBER"
            expect production false "$IRONSTAR_IS_PRODUCTION"
            expect slug refs/pull/42/merge "$IRONSTAR_PREVIEW_SLUG"

            echo "case 3: no id token, tag push (no branch)"
            resolve ""
            expect event push "$IRONSTAR_EVENT"
            expect production false "$IRONSTAR_IS_PRODUCTION"
            expect slug "$shortRev" "$IRONSTAR_PREVIEW_SLUG"

            echo "case 4: id token, push to the default branch"
            start_stub '{"event":"push","ref":"refs/heads/main","repository":"sciexp/ironstar"}'
            resolve main
            expect source id-token "$IRONSTAR_RUN_CONTEXT_SOURCE"
            expect event push "$IRONSTAR_EVENT"
            expect production true "$IRONSTAR_IS_PRODUCTION"
            expect branch main "$IRONSTAR_BRANCH"
            expect slug main "$IRONSTAR_PREVIEW_SLUG"
            stop_stub

            echo "case 5: id token, pull request targeting the default branch"
            echo "        (the eval-time branch is 'main' here: that is the defect)"
            start_stub '{"event":"pull_request","pr_number":42,"base_ref":"refs/heads/main","repository":"sciexp/ironstar"}'
            resolve main
            expect source id-token "$IRONSTAR_RUN_CONTEXT_SOURCE"
            expect event pull_request "$IRONSTAR_EVENT"
            expect pr-number 42 "$IRONSTAR_PR_NUMBER"
            expect production false "$IRONSTAR_IS_PRODUCTION"
            expect branch "" "$IRONSTAR_BRANCH"
            expect slug pr-42 "$IRONSTAR_PREVIEW_SLUG"
            stop_stub

            echo "case 6: id token, push to a non-default branch"
            start_stub '{"event":"push","ref":"refs/heads/topic","repository":"sciexp/ironstar"}'
            resolve topic
            expect event push "$IRONSTAR_EVENT"
            expect production false "$IRONSTAR_IS_PRODUCTION"
            expect slug topic "$IRONSTAR_PREVIEW_SLUG"
            stop_stub

            echo "case 7: id token, scheduled run (no ref claim)"
            start_stub '{"event":"schedule","schedule":"nightly","repository":"sciexp/ironstar"}'
            resolve main
            expect event schedule "$IRONSTAR_EVENT"
            expect production false "$IRONSTAR_IS_PRODUCTION"
            stop_stub

            echo "case 8: id token requested but refused: fail closed, never fall back"
            start_stub '{"event":"pull_request","pr_number":42,"base_ref":"refs/heads/main"}'
            export NIXBOT_ID_TOKEN_REQUEST_TOKEN=wrong-task-token
            if resolve main; then
              echo "  FAIL refused token request resolved anyway"
              failures=$((failures + 1))
            else
              echo "  ok   refused token request aborted the effect"
            fi
            stop_stub

            if [ "$failures" -ne 0 ]; then
              echo ""
              echo "effect-run-context: $failures assertion(s) failed"
              exit 1
            fi

            touch $out
          '';
    };
}
