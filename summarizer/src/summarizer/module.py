import asyncio
import argparse
import sys
from playwright.async_api import async_playwright
from datetime import date
import os

GENERATE_SUMMARY_PROMPT = """
Generate a summary of each source. start each summary with name of source. 
Each source should be summarized in at most 30 words. separate each source by newline.
"""

NOTEBOOKLM_URL = "https://notebooklm.google.com/"


def _is_google_drive_folder(uri):
    return "drive.google.com/drive/folders/" in uri


def _is_arxiv_link(uri):
    return "arxiv.org" in uri


def _is_local_file(uri):
    return os.path.exists(uri) and (uri.endswith('.pdf') or uri.endswith('.md'))


class NotebookLMSummarizer:
    def __init__(self, notebook_url="https://notebooklm.google.com/notebook/6644f7b8-aa26-49a5-afa4-cd16bc312211"):
        self.notebook_url = notebook_url
        self.page = None
        self.playwright = None

    async def get_page(self):
        """Get or create a browser page"""
        if self.page is None:
            self.playwright = await async_playwright().start()
            browser = await self.playwright.chromium.connect_over_cdp("http://localhost:9222")

            contexts = browser.contexts
            if contexts:
                pages = contexts[0].pages
                if pages:
                    self.page = pages[0]
                else:
                    self.page = await contexts[0].new_page()
            else:
                context = await browser.new_context()
                self.page = await context.new_page()

        return self.page

    async def close(self):
        """Clean up Playwright resources"""
        if self.playwright:
            await self.playwright.stop()

    async def initialize_notebook(self):
        """Initialize notebook by opening URL and performing setup"""
        page = await self.get_page()
        await page.goto(NOTEBOOKLM_URL)
        await asyncio.sleep(2)
        await page.click('text=Create new notebook')
        await asyncio.sleep(2)
        return

    async def add_source(self, uri):
        """Main method to add sources from various URI types"""
        page = await self.get_page()
        await asyncio.sleep(10)
        if _is_google_drive_folder(uri):
            return await self._add_google_drive(uri, page)
        elif _is_arxiv_link(uri):
            return await self._add_arxiv(uri, page)
        elif _is_local_file(uri):
            return await self._add_local_file(uri, page)
        else:
            raise ValueError(f"Unsupported URI type: {uri}")

    async def summarize(self):
        """Generate summary from added sources"""
        page = await self.get_page()

        await asyncio.sleep(20)

        textarea_selector = 'textarea[aria-label="Query box"][placeholder="Start typing..."]'
        await page.wait_for_selector(textarea_selector, timeout=10000)
        await page.click(textarea_selector)

        await page.type(textarea_selector, GENERATE_SUMMARY_PROMPT.replace('\n', ''))
        await asyncio.sleep(2)
        await page.keyboard.press('Enter')
        await asyncio.sleep(20)

        last_message = page.locator("chat-message").last
        last_message_text = await last_message.text_content()

        # Clean up the message text
        summary = last_message_text[:-48] if len(last_message_text) > 48 else last_message_text
        return summary

    async def _add_google_drive(self, drive_url, page):
        """Process Google Drive folder and return summary"""
        try:
            await self.open_add_sources_popup(page)

            await page.click('text=Google Docs')

            await asyncio.sleep(8)
            await page.keyboard.press('Tab')
            await asyncio.sleep(1)
            await page.keyboard.press('Tab')

            await asyncio.sleep(1)
            await page.keyboard.type(drive_url)

            await asyncio.sleep(1)
            await page.keyboard.press('Enter')

            await asyncio.sleep(2)
            await page.keyboard.press('Enter')
            await asyncio.sleep(2)

            for i in range(4):
                await page.keyboard.press('Tab')
                await asyncio.sleep(1)

            await page.keyboard.press('Control+a')

            for i in range(2):
                await page.keyboard.press('Tab')
                await asyncio.sleep(1)

            await page.keyboard.press('Enter')
            await asyncio.sleep(20)

        except Exception as e:
            raise Exception(f"Error processing Google Drive folder: {e}")

        return

    async def open_add_sources_popup(self, page):
        # Check if "Add sources" is already visible
        add_sources_visible = await page.locator('text=Add sources').is_visible()
        if not add_sources_visible:
            # Click Add button if "Add sources" is not visible
            await asyncio.sleep(3)
            await page.click('text=Add')
        return

    async def _add_arxiv(self, arxiv_url, page):
        """Process arXiv paper and return summary"""
        try:
            await self.open_add_sources_popup(page)
            await asyncio.sleep(3)
            await page.click('text=Website')
            await asyncio.sleep(2)
            await page.keyboard.press('Tab')
            await asyncio.sleep(1)
            await page.keyboard.press('Tab')
            await asyncio.sleep(1)
            await page.keyboard.type(arxiv_url)
            await asyncio.sleep(1)
            await page.keyboard.press('Tab')
            await asyncio.sleep(1)
            await page.keyboard.press('Enter')


        except Exception as e:
            raise Exception(f"Error processing arXiv url {arxiv_url}: {e}")

        return

    async def _add_local_file(self, file_path, page):
        """Process local PDF/MD file and return summary"""
        try:
            await self.open_add_sources_popup(page)

            await asyncio.sleep(2)

            async with page.expect_file_chooser() as fc_info:
                await page.click('text=choose file')
            file_chooser = await fc_info.value
            await file_chooser.set_files(file_path)
            await asyncio.sleep(1)
        except Exception as e:
            raise Exception(f"Error processing local file: {e}")
        return


async def main():
    parser = argparse.ArgumentParser(description='Summarize documents using NotebookLM')
    parser.add_argument('user_id', help='User ID for the summary')
    parser.add_argument('uris', nargs='+', help='URIs to summarize (local file paths, Google Drive folder URLs, or arXiv URLs)')

    args = parser.parse_args()

    summarizer = NotebookLMSummarizer()

    try:
        await summarizer.initialize_notebook()

        # Process each URI and add as source
        for uri in args.uris:
            print(f"Processing: {uri}")
            if _is_google_drive_folder(uri):
                print(f"  Detected: Google Drive folder")
            elif _is_arxiv_link(uri):
                print(f"  Detected: arXiv URL")
            elif _is_local_file(uri):
                print(f"  Detected: Local file")
            else:
                print(f"  Warning: Unknown URI type, attempting to process anyway")
            
            await summarizer.add_source(uri)
        
        # Generate summary after all sources are added
        print("Generating summary from all sources...")
        summary = await summarizer.summarize()
        
        # Save summary with timestamp
        output_file = f"summary_for_{args.user_id}_on_{date.today()}.md"
        with open(output_file, "w") as f:
            f.write(summary)
            print(f"Summary saved to: {output_file}")

        print(f"\nSummary:\n{summary}")

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    finally:
        await summarizer.close()


if __name__ == "__main__":
    asyncio.run(main())
