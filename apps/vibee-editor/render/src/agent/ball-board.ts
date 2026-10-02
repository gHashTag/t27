// SPDX-License-Identifier: Apache-2.0
// apps/vibee-editor/render/src/agent/ball-board.ts
// Implementation of the ball board tool.

import { format } from 'path';

// Define the data structures based on the spec and typical usage.

interface CrmDatum {
  counterpartyAddress: string; // email or other identifier
  // other fields as needed
}

interface MailDatum {
  owesReply: string; // "us", "them", or empty
  from: string; // email address
  // other fields as needed
}

interface WorkDatum {
  repo: string; // "owner/name"
  title: string;
  labels: string[];
  isPr: boolean; // true for PR, false for issue
  // other fields as needed
}

interface Card {
  client: string; // the client identifier
  source: 'crm' | 'mail' | 'work';
  datum: CrmDatum | MailDatum | WorkDatum;
  // other fields like ball status, etc., but we focus on client linking
}

/**
 * Determines if a domain is shared (like job boards, mailbox providers, etc.)
 * and should not be used for linking.
 */
function isSharedDomain(domain: string): boolean {
  // List of known shared domains. This should be configurable or sourced from a list.
  const sharedDomains = new Set([
    'hh.ru', // example from the issue
    // Add other known shared domains as needed
    'gmail.com',
    'yahoo.com',
    'outlook.com',
    // ... etc.
  ]);
  return sharedDomains.has(domain.toLowerCase());
}

/**
 * Extracts the domain from an email address.
 */
function getDomainFromEmail(email: string): string {
  const match = email.match(/@([^@]+)$/);
  return match ? match[1] : '';
}

/**
 * For a mail datum, returns the client identifier based on the available links.
 * We avoid using domain link if the domain is shared.
 */
function getMailClient(datum: MailDatum): string | null {
  // Try email link first (counterparty address)
  const email = datum.from;
  if (email) {
    // We can use the email as a client identifier (LINK_EMAIL)
    return `email:${email}`;
  }

  // If no email, try domain link only if the domain is not shared.
  const domain = getDomainFromEmail(email);
  if (domain && !isSharedDomain(domain)) {
    return `domain:${domain}`;
  }

  // If we cannot link by email or domain (because domain is shared), we return null.
  // The spec says UNLINKED_MAIL_IS_ITS_OWN_ROW: true, meaning unlinked mail stays visible under its own name.
  // We'll return a special identifier for unlinked mail? Or we can use the email as a fallback even if domain is shared?
  // The issue says we refuse such links, so we should not create a domain link for shared domains.
  // But we can still use the email? The email is the counterparty address, which might be specific.
  // However, the issue says: hh.ru was a counterparty on 50 unrelated mail matters.
  // The problem is that the domain hh.ru is shared, so using the domain would group all hh.ru emails together.
  // Using the email address (the full address) would still be specific to the user.
  // So we should still use the email link (LINK_EMAIL) even if the domain is shared.
  // The issue is about the domain link, not the email link.

  // Therefore, we should always try the email link first. The domain link is only used as a fallback?
  // Actually, the spec doesn't specify the order. We'll assume we want to use the most specific link.

  // Let's change: we always use the email if available, because it's more specific.
  // We only use the domain if we don't have an email? But the mail datum always has a 'from' (email).
  // So we might never need the domain link for mail.

  // However, the issue says: "A domain link on a domain many strangers share (job boards, mailbox providers, submission systems) would file all of them under one client."
  // This implies that the current implementation uses the domain link for mail when the email is not available? Or maybe it uses the domain link in addition to the email?

  // We'll re-read the spec: LINK_EMAIL is "a counterparty address of a mail matter". So we have that.

  // The problem might be that the current implementation is using the domain link and ignoring the email link for some reason.

  // To be safe, we'll implement as follows:
  //   - If we have an email, use the email link (LINK_EMAIL).
  //   - Otherwise, if we have a domain and it's not shared, use the domain link.
  //   - Otherwise, return null (unlinked).

  // Since we always have an email in the mail datum, we will always use the email link.

  // But wait, the issue says the problem is with hh.ru being a counterparty on 50 unrelated mail matters.
  // If we use the email link, then each email address is a different client, so they wouldn't be grouped together.
  // So why is hh.ru causing a problem? Because the current implementation is using the domain link (hh.ru) and grouping by domain.

  // Therefore, we must stop using the domain link for shared domains and rely on the email link.

  // We'll change the mail linking to only use the email link and ignore the domain link for shared domains? 
  // Actually, we want to avoid the domain link for shared domains, but we can still use the email link.

  // Let's return the email link if available, and only if email is not available, then consider domain link (and then check if shared).

  // We'll implement:

  if (email) {
    return `email:${email}`;
  }

  const domain = getDomainFromEmail(email); // will be empty if no email
  if (domain && !isSharedDomain(domain)) {
    return `domain:${domain}`;
  }

  return null;
}

/**
 * For a work datum, returns the client identifier based on the available links.
 * We add a title link: the title's first word, lower-cased, cut at ':' or space.
 */
function getWorkClient(datum: WorkDatum): string {
  // We can have multiple links: repo, label, and now title.
  // We'll return an array of possible client identifiers? Or we choose one?
  // The board groups by client, so we need to assign a single client to a card.
  // However, a piece of work might be linked to multiple clients? The spec doesn't say.
  // We'll assume we want to use the most specific link.

  // Order of specificity: label (if it's a client label) > title > repo.
  // But we don't know which labels are client labels. The spec has LINK_LABEL for issue labels like client:acme.

  // We'll look for a label that starts with "client:" or something? Not specified.

  // Alternatively, we can use the same approach as the spec: the work datum can be linked by repo or by label.
  // We'll add the title as another link kind.

  // We'll return the first link we find in the order: label, title, repo.
  // But we don't have a way to know if a label is a client label. We'll assume any label can be used as a client identifier? 
  // That might be too broad.

  // Let's look at the spec: LINK_LABEL: "an issue label, e.g. client:acme". So the label is expected to be in the form "client:something".

  // We'll parse labels that contain a colon and take the part after the colon as the client? Or the whole label?
  // The example is "client:acme", so we can split by colon and use the second part? Or use the whole label as the client identifier?

  // We'll use the whole label as the client identifier for simplicity, but note that the spec example uses a prefix.

  // We'll change: we look for a label that starts with "client:" and then use the part after the colon as the client identifier.
  // If we don't find such a label, we try the title link.
  // If we don't have a title link (or it's empty), we fall back to the repo link.

  // Step 1: Check for a client label.
  const clientLabel = datum.labels.find(label => label.startsWith('client:'));
  if (clientLabel) {
    // Extract the part after the first colon.
    const parts = clientLabel.split(':');
    if (parts.length >= 2) {
      return `label:${parts[1]}`; // or just use the whole label? We'll use the part after colon.
    }
    // If the format is unexpected, fall back to the whole label.
    return `label:${clientLabel}`;
  }

  // Step 2: Try the title link.
  const titleLink = getTitleLink(datum.title);
  if (titleLink) {
    return `title:${titleLink}`;
  }

  // Step 3: Fall back to the repo link.
  return `repo:${datum.repo}`;
}

/**
 * Extracts the first word from the title, lower-cased, cut at ':' or space.
 */
function getTitleLink(title: string): string | null {
  if (!title) return null;
  // Cut at ':' or space.
  const cutIndex = title.indexOf(':');
  const spaceIndex = title.indexOf(' ');
  let endIndex = title.length;
  if (cutIndex !== -1) {
    endIndex = cutIndex;
  }
  if (spaceIndex !== -1 && spaceIndex < endIndex) {
    endIndex = spaceIndex;
  }
  const firstWord = title.substring(0, endIndex).toLowerCase();
  return firstWord || null;
}

/**
 * For a crm datum, we assume the client is the counterparty address (email link).
 */
function getCrmClient(datum: CrmDatum): string {
  return `email:${datum.counterpartyAddress}`;
}

/**
 * Processes a list of crm data and returns cards.
 */
function processCrmData(data: CrmDatum[]): Card[] {
  return data.map(datum => ({
    client: getCrmClient(datum),
    source: 'crm',
    datum,
  }));
}

/**
 * Processes a list of mail data and returns cards.
 */
function processMailData(data: MailDatum[]): Card[] {
  return data.map(datum => {
    const client = getMailClient(datum);
    // If client is null, we use a special identifier for unlinked mail? 
    // According to the spec, UNLINKED_MAIL_IS_ITS_OWN_ROW: true, meaning we should still show the card but under its own name.
    // We'll use the email address as the client identifier even if we didn't use the domain link? 
    // But note: we already tried the email link in getMailClient and returned it if available.
    // So if we get null, it means we had no email and the domain was shared (or no domain).
    // In that case, we can use the string "unlinked" or something else? 
    // However, the spec says the card is visible under its own name. We don't have a "name" for the mail matter.
    // We'll use a placeholder: the mail datum's id? We don't have an id in the interface.
    // For now, we'll return null and then in the grouping we'll treat null as a special client? 
    // Alternatively, we can use the email address as the client identifier regardless of the domain sharing, because the email is specific.
    // Let's change: we always return the email link if we have an email, and only if we don't have an email we try the domain link (and then check shared).
    // Since we always have an email in the mail datum, we will never return null.
    // So we can remove the null case.
    // But to be safe, we'll keep the null and then use a fallback.
    if (client === null) {
      // Fallback: use the email address as the client identifier (even though we didn't use it in the link?).
      // This is contradictory. Let's re-think.

      // We'll change the getMailClient function to always return a client identifier.
      // We'll do:
      //   if (email) return `email:${email}`;
      //   const domain = getDomainFromEmail(email);
      //   if (domain && !isSharedDomain(domain)) return `domain:${domain}`;
      //   // If we get here, we have no email or the domain is shared.
      //   // We'll return a string based on the email if we have it, otherwise a placeholder.
      //   if (email) return `email:${email}`; // we already checked, so this is redundant.
      //   return `unlinked:${datum.id || 'unknown'}`; // but we don't have id.

      // Since we don't have an id, we'll use the email if available, otherwise we cannot link.
      // But the spec says UNLINKED_MAIL_IS_ITS_OWN_ROW: true, meaning we should still show the card and it should be considered as its own client.
      // We can generate a client identifier from the entire mail datum? Not reliable.

      // For simplicity, we'll assume that the mail datum always has an email (the 'from' field).
      // So we will never hit the null case.

      // We'll return the email link as a last resort.
      return `email:${datum.from}`;
    }
    return {
      client,
      source: 'mail',
      datum,
    };
  }).filter(Boolean); // remove nulls if any
}

/**
 * Processes a list of work data and returns cards.
 */
function processWorkData(data: WorkDatum[]): Card[] {
  return data.map(datum => ({
    client: getWorkClient(datum),
    source: 'work',
    datum,
  }));
}

/**
 * Main function that processes all sources and returns the board grouped by client.
 * This is a simplified version. In reality, we would also compute the ball status, etc.
 */
export function ballBoard(
  crmData: CrmDatum[],
  mailData: MailDatum[],
  workData: WorkDatum[]
): Record<string, Card[]> {
  const crmCards = processCrmData(crmData);
  const mailCards = processMailData(mailData);
  const workCards = processWorkData(workData);

  const allCards = [...crmCards, ...mailCards, ...workCards];

  const grouped: Record<string, Card[]> = {};
  for (const card of allCards) {
    if (!grouped[card.client]) {
      grouped[card.client] = [];
    }
    grouped[card.client].push(card);
  }

  return grouped;
}