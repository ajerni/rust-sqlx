//https://llm-chain.xyz/

use dotenv::dotenv;
use llm_chain::parameters;
use llm_chain::step::Step;
use llm_chain::traits::Executor as ExecutorTrait;
use llm_chain::{chains::sequential::Chain, prompt};
use llm_chain_openai::chatgpt::Executor;

pub async fn run_llm_chain(
    city: String,
    country: String,
) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
    // Create a new ChatGPT executor with default settings
    let exec = Executor::new()?;

    // Create a chain of steps with two prompts
    let chain: Chain = Chain::new(vec![
        // First step: Craft a personalized birthday email / Das erste Argument ist System, das zweite ist User
        Step::for_prompt_template(
            prompt!("You always reply whith a short joke containing the input you received.",
                "Make a joke about {{city}} and {{country}}. Do not use special characters and reply in German.")
        ),
 
        // // Second step: Condense the email into a tweet. Notably, the text parameter takes the output of the previous prompt.
        // Step::for_prompt_template(
        //     prompt!(
        //         "You are an assistant for managing social media accounts for a travel company",
        //         "Format the information into 5 bullet points for the most relevant places. \\\\n--\\\\n{{text}}")
        // ),
    ]);

    // Execute the chain with provided parameters
    let result = chain
        .run(
            // Create a Parameters object with key-value pairs for the placeholders
            parameters!("city" => city, "country" => country),
            &exec,
        )
        .await?;

    let res: String = result.to_immediate().await?.as_content().to_string();

    Ok(res)
}
