use regex::Regex;
use std::sync::OnceLock;

static EMOJI_PATTERNS: OnceLock<Vec<(String, Regex)>> = OnceLock::new();

fn get_emoji_patterns() -> &'static Vec<(String, Regex)> {
    EMOJI_PATTERNS.get_or_init(|| {
        vec![
            ("🇭🇰".to_string(), Regex::new(r"香港|沪港|呼港|中港|HKT|HKBN|HGC|WTT|CMI|穗港|广港|京港|🇭🇰|HK|Hongkong|Hong Kong|HongKong|HONG KONG").unwrap()),
            ("🇹🇼".to_string(), Regex::new(r"台湾|台灣|臺灣|台北|台中|新北|彰化|台|CHT|HINET|TW|Taiwan|TAIWAN").unwrap()),
            ("🇲🇴".to_string(), Regex::new(r"澳门|澳門|(\s|-)?MO\d*|CTM|MAC|Macao|Macau").unwrap()),
            ("🇸🇬".to_string(), Regex::new(r"新加坡|狮城|獅城|沪新|京新|泉新|穗新|深新|杭新|广新|廣新|滬新|SG|Singapore|SINGAPORE").unwrap()),
            ("🇯🇵".to_string(), Regex::new(r"日本|东京|東京|大阪|埼玉|京日|苏日|沪日|广日|上日|穗日|川日|中日|泉日|杭日|深日|JP|Japan|JAPAN").unwrap()),
            ("🇺🇸".to_string(), Regex::new(r"美国|美國|京美|硅谷|凤凰城|洛杉矶|西雅图|圣何塞|芝加哥|哥伦布|纽约|广美|(\s|-)?US\d*|USA|America|United States").unwrap()),
            ("🇰🇷".to_string(), Regex::new(r"韩国|韓國|首尔|首爾|韩|韓|春川|KOR|KR|Kr|Korea").unwrap()),
            ("🇷🇺".to_string(), Regex::new(r"俄罗斯|俄羅斯|毛子|俄国|RU|RUS|Russia").unwrap()),
            ("🇮🇳".to_string(), Regex::new(r"印度|孟买|(\s|-)?IN\d*|IND|India|INDIA|Mumbai").unwrap()),
            ("🇬🇧".to_string(), Regex::new(r"英国|英國|伦敦|UK|England|United Kingdom|Britain").unwrap()),
            ("🇩🇪".to_string(), Regex::new(r"德国|德國|法兰克福|(\s|-)?DE\d*|(\s|-)?GER\d*|🇩🇪|German|GERMAN").unwrap()),
            ("🇫🇷".to_string(), Regex::new(r"法国|法國|巴黎|FR|France").unwrap()),
            ("🇦🇺".to_string(), Regex::new(r"澳大利亚|澳洲|墨尔本|悉尼|(\s|-)?AU\d*|Australia|Sydney").unwrap()),
            ("🇨🇦".to_string(), Regex::new(r"加拿大|蒙特利尔|温哥华|多伦多|多倫多|滑铁卢|楓葉|枫叶|CA|CAN|Waterloo|Canada|CANADA").unwrap()),
            ("🇲🇾".to_string(), Regex::new(r"马来西亚|马来|馬來|MY|Malaysia|MALAYSIA").unwrap()),
            ("🇹🇷".to_string(), Regex::new(r"土耳其|伊斯坦布尔|(\s|-)?TR\d|TR_|TUR|Turkey").unwrap()),
            ("🇵🇭".to_string(), Regex::new(r"菲律宾|菲律賓|(\s|-)?PH\d*|Philippines").unwrap()),
            ("🇹🇭".to_string(), Regex::new(r"泰国|泰國|曼谷|(\s|-)?TH\d*|Thailand").unwrap()),
            ("🇻🇳".to_string(), Regex::new(r"越南|胡志明市|(\s|-)?VN\d*|Vietnam").unwrap()),
            ("🇮🇩".to_string(), Regex::new(r"印尼|印度尼西亚|雅加达|ID|IDN|Indonesia").unwrap()),
            ("🇮🇹".to_string(), Regex::new(r"意大利|義大利|米兰|(\s|-)?IT\d*|Italy|Nachash").unwrap()),
            ("🇪🇸".to_string(), Regex::new(r"西班牙|(\s|-)?ES\d*|Spain").unwrap()),
            ("🇳🇱".to_string(), Regex::new(r"荷兰|荷蘭|尼德蘭|阿姆斯特丹|NL|Netherlands|Amsterdam").unwrap()),
            ("🇵🇱".to_string(), Regex::new(r"波兰|波蘭|(\s|-)?PL\d*|POL|Poland").unwrap()),
            ("🇧🇷".to_string(), Regex::new(r"巴西|圣保罗|维涅杜|BR|Brazil").unwrap()),
            ("🇦🇷".to_string(), Regex::new(r"阿根廷|(\s|-)?AR\d*|Argentina").unwrap()),
            ("🇲🇽".to_string(), Regex::new(r"墨西哥|MX|MEX|MEXICO").unwrap()),
            ("🇿🇦".to_string(), Regex::new(r"南非|约翰内斯堡|(\s|-)?ZA\d*|South Africa|Johannesburg").unwrap()),
            ("🇦🇪".to_string(), Regex::new(r"阿联酋|迪拜|(\s|-)?AE\d*|Dubai|United Arab Emirates").unwrap()),
            ("🇸🇦".to_string(), Regex::new(r"沙特|利雅得|吉达|Saudi|Saudi Arabia").unwrap()),
            ("🇨🇳".to_string(), Regex::new(r"中国|中國|江苏|北京|上海|广州|深圳|杭州|徐州|青岛|宁波|镇江|沈阳|济南|回国|back|(\s|-)?CN\d*|China").unwrap()),
        ]
    })
}

pub fn add_emoji(name: &str) -> String {
    // Check if already starts with emoji
    for (emoji, _) in get_emoji_patterns() {
        if name.starts_with(emoji) {
            return format!("{} {}", emoji, name[emoji.len()..].trim());
        }
    }

    // Find matching pattern
    for (emoji, pattern) in get_emoji_patterns() {
        if pattern.is_match(name) {
            // Handle special case for US emoji 🇺🇲
            if name.starts_with("🇺🇲") {
                return format!("{} {}", emoji, name[4..].trim());
            } else {
                return format!("{} {}", emoji, name);
            }
        }
    }

    name.to_string()
}

pub fn _remove_duplicate_nodes(nodes: &mut Vec<serde_json::Value>) {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    nodes.retain(|node| {
        if let (Some(server), Some(port)) = (node.get("server"), node.get("server_port")) {
            let key = format!("{}:{}", server, port);
            seen.insert(key)
        } else {
            true
        }
    });
}

pub fn _filter_by_keywords(
    nodes: Vec<serde_json::Value>,
    keywords: &str,
    exclude: bool,
) -> Vec<serde_json::Value> {
    if keywords.is_empty() {
        return nodes;
    }

    let patterns: Vec<&str> = keywords.split('|').collect();
    let combined_pattern = patterns.join("|");

    if combined_pattern.is_empty() || combined_pattern.trim().is_empty() {
        return nodes;
    }

    let re = match Regex::new(&combined_pattern) {
        Ok(r) => r,
        Err(_) => return nodes,
    };

    nodes
        .into_iter()
        .filter(|node| {
            if let Some(tag) = node.get("tag").and_then(|t| t.as_str()) {
                let matches = re.is_match(tag);
                if exclude {
                    !matches
                } else {
                    matches
                }
            } else {
                true
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_emoji() {
        assert_eq!(add_emoji("香港节点"), "🇭🇰 香港节点");
        assert_eq!(add_emoji("US Server"), "🇺🇸 US Server");
        assert_eq!(add_emoji("Japan Tokyo"), "🇯🇵 Japan Tokyo");
    }
}
