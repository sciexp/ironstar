/*
  Runtime run-context resolution for the herculesCI effects.

  The effects' production-versus-preview choice used to be taken at nix
  evaluation time from `herculesCI.config.repo.branch`. A build service that
  reports a pull request's branch as its TARGET branch makes that choice
  unsound: a pull request against `main` evaluates with `branch == "main"` and
  selects the production path from unmerged code. The evaluation inputs are
  precisely what the two services disagree about, so no eval-time expression
  can separate the two situations — the decision has to happen inside the
  effect script, where an identity token describing the event is available.

  Dual mode, and the fallback is not optional. When both
  `NIXBOT_ID_TOKEN_REQUEST_URL` and `NIXBOT_ID_TOKEN_REQUEST_TOKEN` are
  present, the run context comes from the token's claims. When either is
  absent — which is every run under the currently authoritative service — it
  comes from the eval-time branch name and reproduces the previous behaviour
  exactly.

  Token mechanism: POST `{"audience":"<declared audience>"}` to the request
  URL with the request token as a bearer credential, take `.token`, read the
  claims from the JWS payload segment. `idTokenAudiences` on the effect
  derivation is what makes the service inject those two variables, so an
  effect that consumes this fragment must declare
  `idTokenAudiences = builtins.toJSON [ effectIdTokenAudience ]` — the value
  must be the JSON-encoded string, since a bare nix list would reach the
  derivation as the space-joined `ironstar-ci` and fail to parse.

  Claim shapes this relies on: `event` is one of `push`, `pull_request`,
  `schedule`; a push token carries `ref = refs/heads/<branch>`; a pull-request
  token carries a numeric `pr_number` and `base_ref` (the target branch) and
  deliberately carries no `ref`, so a pull request has no head-branch name
  available and is identified by its number.

  Two decisions worth stating. Production requires positive evidence of a
  default-branch push, so any event kind other than `push` resolves to
  preview. And when the token path is engaged but cannot be completed, the
  resolver fails rather than falling back, because falling back to the branch
  name is exactly the misclassification the token exists to prevent.
*/
{ ... }:
let
  audience = "ironstar-ci";
  defaultBranch = "main";
in
{
  flake.lib.effectIdTokenAudience = audience;

  flake.lib.mkEffectRunContext =
    pkgs:
    pkgs.writeShellApplication {
      name = "ironstar-effect-run-context";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.curl
        pkgs.jq
      ];
      meta.description = "resolve a herculesCI effect's run context from an id token, or from the eval-time branch name";
      text = ''
        # usage: ironstar-effect-run-context <eval-branch> <eval-short-rev>
        #
        # Prints shell assignments on stdout for the caller to `eval`;
        # diagnostics go to stderr so that capture stays clean.

        audience=${audience}
        default_branch=${defaultBranch}

        eval_branch="''${1-}"
        eval_short_rev="''${2-}"

        fail() {
          printf 'run-context: %s\n' "$1" >&2
          exit 1
        }

        emit() {
          printf '%s=%q\n' "$1" "$2"
        }

        branch_of_ref() {
          case "$1" in
            refs/heads/*) printf '%s' "''${1#refs/heads/}" ;;
            *) printf '%s' "" ;;
          esac
        }

        fallback_slug() {
          if [ -n "$eval_branch" ]; then
            printf '%s' "$eval_branch"
          else
            printf '%s' "$eval_short_rev"
          fi
        }

        id_token_claims() {
          local response token payload
          response="$(
            curl -fsS -X POST "$NIXBOT_ID_TOKEN_REQUEST_URL" \
              -H "Authorization: Bearer $NIXBOT_ID_TOKEN_REQUEST_TOKEN" \
              -H 'Content-Type: application/json' \
              --data "{\"audience\":\"$audience\"}"
          )" || fail "id-token request to $NIXBOT_ID_TOKEN_REQUEST_URL failed"

          token="$(printf '%s' "$response" | jq -re '.token')" ||
            fail 'id-token response carried no .token'

          payload="''${token#*.}"
          payload="''${payload%%.*}"
          if [ -z "$payload" ] || [ "$payload" = "$token" ]; then
            fail 'id token is not a JWS compact serialization'
          fi

          case $((''${#payload} % 4)) in
            0) ;;
            2) payload="''${payload}==" ;;
            3) payload="''${payload}=" ;;
            *) fail 'id token payload is not valid base64url' ;;
          esac

          printf '%s' "$payload" | tr -- '-_' '+/' | base64 -d ||
            fail 'could not decode id token payload'
        }

        if [ -n "''${NIXBOT_ID_TOKEN_REQUEST_URL:-}" ] && [ -n "''${NIXBOT_ID_TOKEN_REQUEST_TOKEN:-}" ]; then
          context_source="id-token"
          claims="$(id_token_claims)"

          event="$(printf '%s' "$claims" | jq -r '.event // ""')"
          ref="$(printf '%s' "$claims" | jq -r '.ref // ""')"
          pr_number="$(printf '%s' "$claims" | jq -r 'if .pr_number == null then "" else (.pr_number | tostring) end')"
          [ -n "$event" ] || fail 'id token carried no event claim'

          branch=""
          case "$event" in
            push)
              branch="$(branch_of_ref "$ref")"
              if [ "$ref" = "refs/heads/$default_branch" ]; then
                is_production=true
              else
                is_production=false
              fi
              if [ -n "$branch" ]; then
                slug="$branch"
              else
                slug="$eval_short_rev"
              fi
              ;;
            pull_request)
              [ -n "$pr_number" ] || fail 'pull_request id token carried no pr_number claim'
              is_production=false
              slug="pr-$pr_number"
              ;;
            *)
              is_production=false
              slug="$(fallback_slug)"
              ;;
          esac
        else
          context_source="branch-name"
          branch="$eval_branch"
          pr_number=""
          event=push
          if [[ "$eval_branch" =~ ^refs/pull/([0-9]+)/merge$ ]]; then
            event=pull_request
            pr_number="''${BASH_REMATCH[1]}"
          fi
          if [ "$eval_branch" = "$default_branch" ]; then
            is_production=true
          else
            is_production=false
          fi
          slug="$(fallback_slug)"
        fi

        emit IRONSTAR_RUN_CONTEXT_SOURCE "$context_source"
        emit IRONSTAR_EVENT "$event"
        emit IRONSTAR_PR_NUMBER "$pr_number"
        emit IRONSTAR_IS_PRODUCTION "$is_production"
        emit IRONSTAR_BRANCH "$branch"
        emit IRONSTAR_PREVIEW_SLUG "$slug"
      '';
    };
}
