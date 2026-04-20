import { drizzle } from "drizzle-orm/node-postgres";
import { Pool } from "pg";

import { config } from "../config.js";

export const pool = new Pool({
  connectionString: config.databaseUrl,
  max: 10,
  idleTimeoutMillis: 30_000,
  connectionTimeoutMillis: 5_000,
});

export const db = drizzle(pool);
