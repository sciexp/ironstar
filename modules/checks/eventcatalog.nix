{ ... }:
{
  perSystem =
    { config, ... }:
    {
      checks.ironstar-eventcatalog-unit = config.packages.ironstar-eventcatalog.tests.unit;
      checks.ironstar-eventcatalog-e2e = config.packages.ironstar-eventcatalog.tests.e2e;
    };
}
