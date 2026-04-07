{ ... }:
{
  perSystem =
    { config, ... }:
    {
      checks.ironstar-docs-unit = config.packages.ironstar-docs.tests.unit;
      checks.ironstar-docs-e2e = config.packages.ironstar-docs.tests.e2e;
    };
}
