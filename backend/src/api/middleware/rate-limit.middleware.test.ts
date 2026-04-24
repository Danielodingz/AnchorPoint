import { Request, Response, NextFunction } from 'express';
import { apiKeyMiddleware } from './api-key.middleware';
import { burstRateLimiter, sustainedRateLimiter } from './rate-limit.middleware';
import prisma from '../../lib/prisma';
import { redis } from '../../lib/redis';

// Mock dependencies
jest.mock('../../lib/prisma', () => ({
  __esModule: true,
  default: {
    apiKey: {
      findUnique: jest.fn(),
    },
  },
}));

jest.mock('../../lib/redis', () => ({
  redis: {
    get: jest.fn(),
    setex: jest.fn(),
    call: jest.fn().mockResolvedValue('mock-sha-string'), // for rate-limit-redis
  },
}));

describe('Rate Limiting & API Key Middleware', () => {
  let req: Partial<Request>;
  let res: Partial<Response>;
  let next: NextFunction;

  beforeEach(() => {
    req = {
      header: jest.fn().mockReturnValue(undefined),
      ip: '127.0.0.1',
    };
    res = {
      status: jest.fn().mockReturnThis(),
      json: jest.fn(),
      send: jest.fn(),
    };
    next = jest.fn();
    jest.clearAllMocks();
  });

  describe('apiKeyMiddleware', () => {
    it('should default to FREE tier if no API key is provided', async () => {
      await apiKeyMiddleware(req as Request, res as Response, next);
      expect(req.apiTier).toBe('FREE');
      expect(next).toHaveBeenCalled();
    });

    it('should use cached tier if available in Redis', async () => {
      req.header = jest.fn().mockReturnValue('cached-key');
      (redis.get as jest.Mock).mockResolvedValue('PREMIUM');

      await apiKeyMiddleware(req as Request, res as Response, next);

      expect(req.apiKey).toBe('cached-key');
      expect(req.apiTier).toBe('PREMIUM');
      expect(next).toHaveBeenCalled();
    });

    it('should lookup in Postgres if not cached', async () => {
      req.header = jest.fn().mockReturnValue('db-key');
      (redis.get as jest.Mock).mockResolvedValue(null);
      (prisma.apiKey.findUnique as jest.Mock).mockResolvedValue({ tier: 'BASIC' });

      await apiKeyMiddleware(req as Request, res as Response, next);

      expect(prisma.apiKey.findUnique).toHaveBeenCalledWith({ where: { key: 'db-key' }, select: { tier: true } });
      expect(redis.setex).toHaveBeenCalledWith('apikey:db-key', 300, 'BASIC');
      expect(req.apiKey).toBe('db-key');
      expect(req.apiTier).toBe('BASIC');
      expect(next).toHaveBeenCalled();
    });

    it('should return 401 if API key is invalid', async () => {
      req.header = jest.fn().mockReturnValue('invalid-key');
      (redis.get as jest.Mock).mockResolvedValue(null);
      (prisma.apiKey.findUnique as jest.Mock).mockResolvedValue(null);

      await apiKeyMiddleware(req as Request, res as Response, next);

      expect(res.status).toHaveBeenCalledWith(401);
      expect(res.json).toHaveBeenCalledWith({ error: 'Invalid API Key' });
      expect(next).not.toHaveBeenCalled();
    });
  });

  describe('rate limiters setup', () => {
    it('burstRateLimiter should be defined', () => {
      expect(burstRateLimiter).toBeDefined();
    });

    it('sustainedRateLimiter should be defined', () => {
      expect(sustainedRateLimiter).toBeDefined();
    });
  });
});
