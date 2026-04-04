{ ... }:
{
  perSystem =
    { config, ... }:
    {
      checks.docs-unit = config.packages.ironstar-docs.tests.unit;
      checks.docs-e2e = config.packages.ironstar-docs.tests.e2e;
    };
}
