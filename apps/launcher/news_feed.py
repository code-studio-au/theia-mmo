"""Read public article summaries from the Theia Online RSS feed."""
from dataclasses import dataclass
from http.client import HTTPException
import re
from urllib.request import Request, build_opener
from xml.etree import ElementTree

from hosted_release import NoRedirect

NEWS_URL = 'https://theiaonline.com/news/'
FEED_URL = 'https://theiaonline.com/rss.xml'
MAX_FEED_BYTES = 256 * 1024
MAX_ARTICLES = 20


@dataclass(frozen=True)
class NewsArticle:
    title: str
    summary: str
    url: str


def parse_news(data):
    if len(data) > MAX_FEED_BYTES:
        raise ValueError('News feed exceeds its size limit')
    if re.search(rb'<!\s*(DOCTYPE|ENTITY)', data, flags=re.IGNORECASE):
        raise ValueError('Unexpected declaration in news feed')
    try:
        feed = ElementTree.fromstring(data.decode('utf-8-sig'))
    except (UnicodeError, ElementTree.ParseError) as error:
        raise ValueError('Invalid news feed') from error
    if feed.tag != 'rss' or feed.find('channel') is None:
        raise ValueError('Invalid news feed')

    articles = []
    seen = set()
    for item in feed.findall('./channel/item'):
        title = ' '.join(item.findtext('title', '').split())
        summary = ' '.join(item.findtext('description', '').split())
        url = item.findtext('link', '').strip()
        if (not title or url in seen
                or not re.fullmatch(re.escape(NEWS_URL) + r'[a-z0-9][a-z0-9-]{0,119}/', url)):
            continue
        articles.append(NewsArticle(title[:120], summary[:320], url))
        seen.add(url)
        if len(articles) == MAX_ARTICLES:
            break
    return articles


def fetch_news():
    request = Request(FEED_URL, headers={'Accept': 'application/rss+xml', 'User-Agent': 'TheiaLauncher'})
    try:
        with build_opener(NoRedirect()).open(request, timeout=8) as response:
            return parse_news(response.read(MAX_FEED_BYTES + 1))
    except HTTPException as error:
        raise OSError('Could not read Theia Online news') from error
