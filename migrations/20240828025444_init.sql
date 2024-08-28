CREATE SCHEMA partman;
CREATE EXTENSION pg_partman WITH SCHEMA partman;

CREATE SCHEMA ge;
CREATE TABLE ge.price(
    created timestamp NOT NULL DEFAULT (now() at time zone 'utc'),
    item BIGINT NOT NULL,
    high DECIMAL,
    high_time BIGINT,
    low DECIMAL,
    low_time BIGINT
    )PARTITION BY RANGE (created);

SELECT partman.create_parent( p_parent_table => 'ge.price',
 p_control => 'created',
 p_interval=> '1 month',
 p_premake => 2);

CREATE INDEX ON ge.price(created DESC, item);
CREATE INDEX ON ge.price(created);


