import { rateLimit, Options as RateLimitOptionsBase } from 'express-rate-limit';
import RedisStore from 'rate-limit-redis';
import { redis } from '../../lib/redis';
import logger from '../../utils/logger';
import { Request, Response, NextFunction } from 'express';

export interface RateLimitOptions {
  windowMs?: number;
  max?: number | ((req: Request, res: Response) => number | Promise<number>);
  message?: string;
  keyPrefix?: string;
}

export const createRateLimiter = (options: RateLimitOptions = {}) => {
  const {
    windowMs = 15 * 60 * 1000,
    max = 100,
    message = 'Too many requests, please try again later.',
    keyPrefix = 'rl:',
  } = options;

  return rateLimit({
    windowMs,
    max,
    message: { error: message },
    standardHeaders: true,
    legacyHeaders: false,
    keyGenerator: (req: Request) => {
      return req.apiKey ? req.apiKey : req.ip || 'unknown';
    },
    store: new RedisStore({
      // @ts-expect-error - ioredis call signature mismatch
      sendCommand: (...args: string[]) => redis.call(...args),
      prefix: keyPrefix,
    }),
    handler: (req: Request, res: Response, _next: NextFunction, options: any) => {
      logger.warn(`Rate limit exceeded for ${req.apiKey ? 'API Key' : 'IP'}: ${req.apiKey || req.ip} (Tier: ${req.apiTier || 'UNKNOWN'})`);
      res.status(options.statusCode).send(options.message);
    },
  });
};

const getBurstLimit = (tier?: string) => {
  switch (tier) {
    case 'PREMIUM': return 100; // 100 req per minute
    case 'BASIC': return 50;    // 50 req per minute
    default: return 10;         // 10 req per minute
  }
};

const getSustainedLimit = (tier?: string) => {
  switch (tier) {
    case 'PREMIUM': return 1000; // 1000 req per hour
    case 'BASIC': return 500;    // 500 req per hour
    default: return 100;         // 100 req per hour
  }
};

export const burstRateLimiter = createRateLimiter({
  windowMs: 60 * 1000, // 1 minute
  max: (req: Request) => getBurstLimit(req.apiTier),
  message: 'Burst rate limit exceeded, please slow down.',
  keyPrefix: 'rl:burst:',
});

export const sustainedRateLimiter = createRateLimiter({
  windowMs: 60 * 60 * 1000, // 1 hour
  max: (req: Request) => getSustainedLimit(req.apiTier),
  message: 'Sustained rate limit exceeded, please try again later.',
  keyPrefix: 'rl:sustained:',
});

// Keep existing limiters for backwards compatibility if needed, or adjust them
export const apiLimiter = createRateLimiter({
  windowMs: 15 * 60 * 1000,
  max: 1000,
  keyPrefix: 'rl:api:',
});

export const authLimiter = createRateLimiter({
  windowMs: 10 * 60 * 1000,
  max: 10,
  message: 'Too many authentication attempts, please try again after 10 minutes.',
  keyPrefix: 'rl:auth:',
});

export const sensitiveApiLimiter = createRateLimiter({
  windowMs: 60 * 1000,
  max: 5,
  message: 'Too many requests to this sensitive endpoint, please try again later.',
  keyPrefix: 'rl:sensitive:',
});
