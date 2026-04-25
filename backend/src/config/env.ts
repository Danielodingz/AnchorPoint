import { z } from 'zod';
import dotenv from 'dotenv';

dotenv.config();

const inferredNodeEnv = process.env.NODE_ENV === 'test' ? 'test' : process.env.NODE_ENV;

const staticEnvSchema = z.object({
  NODE_ENV: z.enum(['development', 'production', 'test']).default('development'),
  PORT: z
    .string()
    .default('3002')
    .transform((val: string) => parseInt(val, 10))
    .pipe(z.number().positive()),
  DATABASE_URL: z.string().min(1, 'DATABASE_URL is required').default('postgresql://postgres:postgres@localhost:5432/anchorpoint?schema=public'),
  REDIS_URL: z.string().url().default('redis://localhost:6379'),
});

export const dynamicConfigSchema = z.object({
  JWT_SECRET: z.string().min(8, 'JWT_SECRET must be at least 8 characters').default('stellar-anchor-secret'),
  INTERACTIVE_URL: z.string().url().default('http://localhost:3000'),
  WEBHOOK_URL: z.string().url().optional(),
  WEBHOOK_SECRET: z.string().min(1, 'WEBHOOK_SECRET cannot be empty').optional(),
<<<<<<< HEAD
  WEBHOOK_TIMEOUT_MS: z
    .string()
    .default('5000')
    .transform((val: string) => parseInt(val, 10))
    .pipe(z.number().positive()),
  WEBHOOK_MAX_RETRIES: z
    .string()
    .default('3')
    .transform((val: string) => parseInt(val, 10))
    .pipe(z.number().int().min(0).max(10)),
  WEBHOOK_RETRY_DELAY_MS: z
    .string()
    .default('500')
    .transform((val: string) => parseInt(val, 10))
    .pipe(z.number().int().min(0)),
  JAEGER_ENDPOINT: z.string().url().default('http://localhost:14268/api/traces'),
  PROMETHEUS_METRICS_PORT: z
    .string()
    .default('9464')
    .transform((val: string) => parseInt(val, 10))
    .pipe(z.number().positive()),
  OTEL_SERVICE_NAME: z.string().default('anchorpoint-backend'),
  OTEL_RESOURCE_ATTRIBUTES: z.string().optional(),
=======
  WEBHOOK_TIMEOUT_MS: z.coerce.number().positive().default(5000),
  WEBHOOK_MAX_RETRIES: z.coerce.number().int().min(0).max(10).default(3),
  WEBHOOK_RETRY_DELAY_MS: z.coerce.number().int().min(0).default(500),
>>>>>>> pr-190
});

export type DynamicConfig = z.infer<typeof dynamicConfigSchema>;

const parsedStatic = staticEnvSchema.safeParse({
  ...process.env,
  NODE_ENV: inferredNodeEnv,
});

if (!parsedStatic.success) {
  console.error('Invalid static environment variables:\n', parsedStatic.error.flatten().fieldErrors);
  process.exit(1);
}

export const staticConfig = parsedStatic.data;

// We also parse the dynamic config from process.env to serve as initial seed values
const parsedInitialDynamic = dynamicConfigSchema.safeParse(process.env);
export const initialDynamicConfig = parsedInitialDynamic.success ? parsedInitialDynamic.data : dynamicConfigSchema.parse({});

// For backward compatibility while refactoring
export const config = {
  ...staticConfig,
  ...initialDynamicConfig,
};
export type Config = typeof config;
