# mkStructuralCheck: runCommand JSON-diff helper for asserting that a
# nix value (typically `attrNames` of some attrset, or a sorted list of
# strings) matches a literal expectation.
#
# Inputs and outputs are JSON-serialized at outer eval time and embedded
# as `passAsFile` env vars, so the derivation's input-address closure
# scopes to "did the JSON change?" rather than "did any tracked file in
# the repo change?". The architectural payoff for structural assertions:
# cache invalidation tracks the assertion target, not `inputs.self`.
#
# Failure mode emits a unified diff, which is readable for set-equality
# failures.
#
# Ported verbatim from vanixiets commit 3fd31c0fb
# (modules/lib/mk-structural-check.nix).
{ ... }:
{
  flake.lib.mkStructuralCheck =
    pkgs:
    {
      name,
      actual,
      expected,
    }:
    pkgs.runCommand "structure-${name}"
      {
        actualJson = builtins.toJSON actual;
        expectedJson = builtins.toJSON expected;
        passAsFile = [
          "actualJson"
          "expectedJson"
        ];
        meta.description = "structural check: ${name}";
      }
      ''
        if ! diff -u "$expectedJsonPath" "$actualJsonPath"; then
          echo ""
          echo "structural check '${name}' failed: actual differs from expected"
          exit 1
        fi
        touch $out
      '';
}
