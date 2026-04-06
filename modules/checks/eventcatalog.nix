{ ... }:
{
  perSystem =
    { config, ... }:
    {
      checks.eventcatalog-unit = config.packages.ironstar-eventcatalog.tests.unit;
      checks.eventcatalog-e2e = config.packages.ironstar-eventcatalog.tests.e2e;
    };
}
