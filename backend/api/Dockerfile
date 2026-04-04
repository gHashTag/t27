FROM node:22-alpine

WORKDIR /app

RUN corepack enable && corepack prepare npm@latest --activate

COPY package.json ./
COPY tsconfig.json ./
COPY drizzle.config.ts ./

RUN npm install

COPY src ./src

RUN npm run build

EXPOSE 3000

CMD ["node", "dist/index.js"]
