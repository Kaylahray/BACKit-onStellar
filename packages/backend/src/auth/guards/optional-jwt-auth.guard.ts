import { Injectable, CanActivate, ExecutionContext } from '@nestjs/common';
import { AuthService } from '../auth.service';

/**
 * Like {@link JwtAuthGuard} but never rejects the request. When a valid
 * `Bearer` token is present it populates `request.user = { address }`;
 * otherwise the request proceeds anonymously (no `request.user`).
 *
 * Used for public endpoints whose response is enriched for authenticated
 * callers (e.g. adding `isBookmarked` to calls) while remaining accessible to
 * anonymous visitors.
 */
@Injectable()
export class OptionalJwtAuthGuard implements CanActivate {
  constructor(private readonly authService: AuthService) {}

  canActivate(context: ExecutionContext): boolean {
    const request = context.switchToHttp().getRequest<{
      headers: Record<string, string | undefined>;
      user?: { address: string };
    }>();
    const authHeader = request.headers['authorization'];

    if (authHeader?.startsWith('Bearer ')) {
      try {
        const payload = this.authService.validateToken(authHeader.slice(7));
        request.user = { address: payload.sub };
      } catch {
        // Invalid/expired token → treat as anonymous; do not throw.
      }
    }

    return true;
  }
}
