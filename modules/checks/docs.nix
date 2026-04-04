{ ... }:
{
  perSystem =
    { config, ... }:
    {
      checks.docs-e2e = config.packages.ironstar-docs.tests.e2e;
    };
}
