FROM node:20-bookworm-slim AS builder
WORKDIR /app/app.smoke/web
RUN corepack enable
ARG GITHUB_PACKAGES_TOKEN=
ARG VITE_ENV_LABEL=compose
ENV VITE_ENV_LABEL=$VITE_ENV_LABEL

COPY app.smoke/web/package.json app.smoke/web/pnpm-lock.yaml* ./
RUN if [ -n "$GITHUB_PACKAGES_TOKEN" ]; then printf '%s\n%s\n' "@liberte-top:registry=https://npm.pkg.github.com" "//npm.pkg.github.com/:_authToken=$GITHUB_PACKAGES_TOKEN" > /root/.npmrc; fi
RUN --mount=type=cache,id=pnpm-store,target=/root/.local/share/pnpm/store pnpm install --no-frozen-lockfile

COPY app.smoke/web ./
RUN pnpm build

FROM nginx:1.27-alpine AS runtime
COPY app.smoke/web/docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/app.smoke/web/dist /usr/share/nginx/html

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
