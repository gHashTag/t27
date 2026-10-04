// SPDX-License-Identifier: Apache-2.0
// apps/vibee-editor/render/src/agent/chat.ts -- marketplace agent prompt and system prompt for v6: client work carries the client's marks
// Host: gHashTag/999-multibots-telegraf, implements marketplaceAgentPrompt, systemPrompt, and runAgent functions

import { 
    CONTENT_FOR_FACE_OF, 
    CONTENT_FOR_SELLER_SWEEP, 
    CLIENT_WORK_MARKS,
    BLOGGER_BRIEF_IS_HOUSE,
    SELLER_PLAN_IN_PROMPT 
} from '../../../../specs/automation/crm-client-workspace.t27';

import { 
    isClientWork, 
    shouldUseClientBriefForBlogger, 
    shouldExcludeSellerPlanFromPrompt,
    shouldSpeakClientWords,
    buildClientWorkContext 
} from '../../../src/agent/client-work';

/**
 * Marketplace agent prompt builder - constructs the prompt based on client work rules.
 * Handles the logic for what content to include based on whether it's client work.
 */
export function marketplaceAgentPrompt(
    contentFor: string,
    bloggerBrief: any,
    sellerPlan: any,
    clientProfile: any,
    hasBloggerChosen: boolean = false
): string {
    const clientWorkContext = buildClientWorkContext(contentFor, hasBloggerChosen, !!sellerPlan);
    
    let prompt = "You are a marketplace agent helping sellers create content. ";
    
    // Add client work context
    if (clientWorkContext.isClientWork) {
        prompt += "This is client work - you are creating content for a specific client. ";
        
        // Handle blogger brief based on client work rules
        if (hasBloggerChosen && clientWorkContext.useClientBrief) {
            prompt += "Use the client's brief for the Blogger. ";
            if (bloggerBrief) {
                prompt += `Client brief: ${JSON.stringify(bloggerBrief)}. `;
            }
        } else if (hasBloggerChosen && !clientWorkContext.useClientBrief) {
            prompt += "Do NOT use the house's brief for the Blogger. ";
        }
        
        // Handle seller plan exclusion
        if (clientWorkContext.excludeSellerPlan && sellerPlan) {
            prompt += "Exclude the seller's plan and post voice from the prompt. ";
        }
    } else {
        prompt += "This is general work - you are creating content for the seller. ";
        
        // Use seller's brief for general work
        if (bloggerBrief) {
            prompt += `Use the seller's brief: ${JSON.stringify(bloggerBrief)}. `;
        }
    }
    
    // Add client profile if available
    if (clientProfile) {
        prompt += `Client profile: ${JSON.stringify(clientProfile)}. `;
    }
    
    // Always include the instruction about speaking words in seller's name
    if (clientWorkContext.speakClientWords) {
        prompt += "Speak the words sent in the seller's name, for example in the duet. ";
    }
    
    return prompt;
}

/**
 * System prompt that incorporates client work rules.
 * Provides the overarching system context for the agent.
 */
export function systemPrompt(
    isClientWork: boolean,
    hasBloggerChosen: boolean = false,
    hasSellerPlan: boolean = false
): string {
    let prompt = "You are an AI assistant helping sellers create marketplace content. ";
    
    if (isClientWork) {
        prompt += "CLIENT WORK MODE: You are working for a specific client. ";
        prompt += `Valid client work marks: ${CLIENT_WORK_MARKS}. `;
        
        if (hasBloggerChosen) {
            prompt += `BLOGGER MODE: ${shouldUseClientBriefForBlogger() ? 
                "Use the client's brief, never the house's." : 
                "Do NOT use the house's brief."} `;
        }
        
        if (hasSellerPlan) {
            prompt += `${shouldExcludeSellerPlanFromPrompt() ? 
                "EXCLUDE the seller's plan and post voice from the prompt." : 
                "Include the seller's plan and post voice in the prompt."} `;
        }
        
        prompt += "SOUL MODE: You still speak the words sent in the seller's name, for example in the duet. ";
    } else {
        prompt += "GENERAL WORK MODE: You are working for the seller directly. ";
        prompt += "Use the seller's brief, plan, and post voice as needed. ";
    }
    
    return prompt;
}

/**
 * Runs the agent with client work detection and rules.
 * This is the main function that orchestrates the agent's behavior.
 */
export function runAgent(
    contentFor: string,
    bloggerBrief: any,
    sellerPlan: any,
    clientProfile: any,
    hasBloggerChosen: boolean = false,
    userMessage: string = ""
): Promise<string> {
    return new Promise((resolve) => {
        // Determine if this is client work
        const isClientWorkTurn = isClientWork(contentFor);
        
        // Build appropriate prompts
        const marketPrompt = marketplaceAgentPrompt(
            contentFor, 
            bloggerBrief, 
            sellerPlan, 
            clientProfile, 
            hasBloggerChosen
        );
        
        const sysPrompt = systemPrompt(isClientWorkTurn, hasBloggerChosen, !!sellerPlan);
        
        // Simulate agent processing - in a real implementation, this would call an LLM
        const agentResponse = processAgentRequest(
            marketPrompt,
            sysPrompt,
            userMessage,
            isClientWorkTurn,
            hasBloggerChosen,
            !!sellerPlan
        );
        
        resolve(agentResponse);
    });
}

/**
 * Internal function to simulate agent processing.
 * In a real implementation, this would interface with an LLM service.
 */
function processAgentRequest(
    marketPrompt: string,
    systemPrompt: string,
    userMessage: string,
    isClientWork: boolean,
    hasBloggerChosen: boolean,
    hasSellerPlan: boolean
): string {
    // This is a simulation - in production, this would call an LLM
    let response = "Agent response: ";
    
    if (isClientWork) {
        response += "Processing client work with client's marks. ";
        
        if (hasBloggerChosen) {
            response += shouldUseClientBriefForBlogger() ? 
                "Using client's brief for Blogger." : 
                "Excluding house's brief for Blogger.";
        }
        
        if (hasSellerPlan) {
            response += shouldExcludeSellerPlanFromPrompt() ? 
                "Excluding seller's plan from prompt." : 
                "Including seller's plan in prompt.";
        }
    } else {
        response += "Processing general work. ";
    }
    
    response += `User message: ${userMessage}`;
    
    return response;
}

/**
 * Helper function to validate content_for marks.
 */
export function validateContentFor(contentFor: string): boolean {
    return isClientWork(contentFor) || contentFor === "general";
}

/**
 * Gets the description of client work rules for debugging/logging.
 */
export function getClientWorkRulesDescription(): string {
    return `
Client Work Rules (v6):
- Client work marks: ${CLIENT_WORK_MARKS}
- Blogger brief: ${shouldUseClientBriefForBlogger() ? "client's brief" : "house's brief"}
- Seller plan in prompt: ${!shouldExcludeSellerPlanFromPrompt() ? "included" : "excluded"}
- SOUL speaks client words: ${shouldSpeakClientWords()}
    `;
}