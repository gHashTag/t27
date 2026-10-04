// SPDX-License-Identifier: Apache-2.0
// src/agent/client-work.ts -- client work detection and rules for v6: client work carries the client's marks
// Host: gHashTag/999-multibots-telegraf, implements the onClientWork function for the marketplace agent prompt

import { 
    CONTENT_FOR_FACE_OF, 
    CONTENT_FOR_SELLER_SWEEP, 
    CLIENT_WORK_MARKS,
    BLOGGER_BRIEF_IS_HOUSE,
    SELLER_PLAN_IN_PROMPT 
} from '../specs/automation/crm-client-workspace.t27';

/**
 * Determines if a turn is client work based on the content_for marks.
 * A turn is client work when it carries faceOf (client thread, or the duet's buyer) 
 * or sellerSweep (an unattended sweep over the seller's leads). These are the only 
 * two marks a turn has (content_for).
 */
export function isClientWork(contentFor: string): boolean {
    const clientWorkMarks = CLIENT_WORK_MARKS.split(',');
    return clientWorkMarks.includes(contentFor);
}

/**
 * Gets the client work marks for validation.
 * Returns the two valid marks: faceOf and sellerSweep.
 */
export function getClientWorkMarks(): string[] {
    return CLIENT_WORK_MARKS.split(',');
}

/**
 * Validates that a content_for mark is valid for client work.
 */
export function isValidClientWorkMark(contentFor: string): boolean {
    return isClientWork(contentFor);
}

/**
 * Determines if the Blogger should get the client's brief (never the house's).
 * On client work: the Blogger gets the client's brief, never the house's.
 */
export function shouldUseClientBriefForBlogger(): boolean {
    return !BLOGGER_BRIEF_IS_HOUSE;
}

/**
 * Determines if the seller's plan and post voice should be excluded from the prompt.
 * On client work: the seller's plan and post voice stay out of the prompt.
 */
export function shouldExcludeSellerPlanFromPrompt(): boolean {
    return !SELLER_PLAN_IN_PROMPT;
}

/**
 * Determines if the SOUL should speak the words sent in the seller's name.
 * The SOUL still speaks the words sent in his name, for example in the duet.
 */
export function shouldSpeakClientWords(): boolean {
    return true; // Always true - the SOUL speaks the words sent in the seller's name
}

/**
 * Builds client work context for the marketplace agent prompt.
 * This function is called to determine what context should be included
 * when the agent is processing client work.
 */
export function buildClientWorkContext(contentFor: string, hasBloggerChosen: boolean, hasSellerPlan: boolean) {
    const isClientWorkTurn = isClientWork(contentFor);
    
    return {
        isClientWork: isClientWorkTurn,
        useClientBrief: isClientWorkTurn && hasBloggerChosen && shouldUseClientBriefForBlogger(),
        excludeSellerPlan: isClientWorkTurn && shouldExcludeSellerPlanFromPrompt(),
        speakClientWords: shouldSpeakClientWords(),
        validMarks: getClientWorkMarks(),
        contentFor: contentFor
    };
}

/**
 * Legacy function name for backward compatibility - maps to isClientWork
 * @deprecated Use isClientWork instead
 */
export function onClientWork(contentFor: string): boolean {
    return isClientWork(contentFor);
}