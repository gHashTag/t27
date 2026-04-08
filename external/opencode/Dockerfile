# Direct TypeScript compilation without pnpm cache issues
FROM node:22-alpine

WORKDIR /app/packages/api

COPY packages/api/package.json ./
COPY packages/api/tsconfig.json ./
COPY packages/api/src ./

RUN npx -y typescript@5.9.3 tsc

EXPOSE 3000

CMD ["node", "dist/index.js"]
