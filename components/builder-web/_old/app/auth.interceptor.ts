import {
  HttpEvent,
  HttpInterceptor,
  HttpHandler,
  HttpRequest,
} from "@angular/common/http";
import { Observable } from "rxjs/observable";
import { AuthService } from "./auth.service";

export class AuthInterceptor implements HttpInterceptor {
  constructor(public auth: AuthService) {}

  intercept(req: HttpRequest<any>, next: HttpHandler): Observable<HttpEvent<any>> {
    if (this.auth.isAuthenticated) {
      const clonedRequest = req.clone({
        headers: req.headers.set("Authorization", `Bearer ${this.auth.token}`)
      });
      return next.handle(clonedRequest);
    }
    return next.handle(req);
  }
}
