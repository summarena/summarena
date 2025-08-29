cargo run --bin test_reranking -- --count 50 --dataset fiqa
FINAL COMPARISON RESULTS
============================================================

Query 1: What does a high theta mean for an option position?
Baseline NDCG@10: 0.3155
  Persona: A seasoned options trader at a hedge fund, Alex ha... → NDCG@10: 0.3869 (Δ0.0714)
  Persona: A finance student in her final year, Jessica is ea... → NDCG@10: 0.5000 (Δ0.1845)
  Persona: An individual investor and part-time blogger, Mark... → NDCG@10: 0.4307 (Δ0.1152)
  Persona: A cryptocurrency enthusiast and self-proclaimed "a... → NDCG@10: 0.4307 (Δ0.1152)

Query 2: How come we can find stocks with a Price-to-Book ratio less ...
Baseline NDCG@10: 0.7668
  Persona: A seasoned financial analyst at a hedge fund, this... → NDCG@10: 0.7537 (Δ-0.0131)
  Persona: An undergraduate student majoring in finance, this... → NDCG@10: 0.7877 (Δ0.0209)
  Persona: A retail investor with years of personal trading e... → NDCG@10: 0.7537 (Δ-0.0131)
  Persona: A conspiracy theorist who believes that traditiona... → NDCG@10: 0.7366 (Δ-0.0302)

Query 3: What does it mean if “IPOs - normally are sold with an `unde...
Baseline NDCG@10: 0.9197
  Persona: A junior financial analyst at a major investment f... → NDCG@10: 0.9197 (Δ0.0000)
  Persona: A seasoned venture capitalist with a strong backgr... → NDCG@10: 0.9197 (Δ0.0000)
  Persona: A finance professor researching the effects of und... → NDCG@10: 0.6509 (Δ-0.2688)
  Persona: A retail investor with a personal finance blog, fr... → NDCG@10: 0.9197 (Δ0.0000)

Query 4: What margin is required to initiate and maintain a short sal...
Baseline NDCG@10: 0.5000
  Persona: An experienced financial analyst working at a hedg... → NDCG@10: 0.6309 (Δ0.1309)
  Persona: A retail investor with a background in engineering... → NDCG@10: 0.6309 (Δ0.1309)
  Persona: A compliance officer in a brokerage firm, she is i... → NDCG@10: 0.6309 (Δ0.1309)
  Persona: A casual investor who believes that short selling ... → NDCG@10: 0.6309 (Δ0.1309)

Query 5: Is it ever a good idea to close credit cards?
Baseline NDCG@10: 0.6372
  Persona: A recent college graduate working as an entry-leve... → NDCG@10: 0.6274 (Δ-0.0097)
  Persona: A small business owner who has several credit card... → NDCG@10: 0.6968 (Δ0.0597)
  Persona: A financial advisor with over a decade of experien... → NDCG@10: 0.7066 (Δ0.0694)
  Persona: A self-proclaimed personal finance guru who believ... → NDCG@10: 0.6521 (Δ0.0149)

Query 6: What do these numbers mean? (futures)
Baseline NDCG@10: 0.2961
  Persona: A seasoned financial analyst at a large investment... → NDCG@10: 0.6508 (Δ0.3547)
  Persona: A recent graduate with a degree in economics, Davi... → NDCG@10: 0.4373 (Δ0.1413)
  Persona: Sarah, a self-employed commodities trader, has bee... → NDCG@10: 0.3827 (Δ0.0866)
  Persona: Mark is a conspiracy theorist who believes that tr... → NDCG@10: 0.2021 (Δ-0.0940)

Query 7: What is the proper way to report additional income for taxes...
Baseline NDCG@10: 0.9469
  Persona: A freelance Android developer with five years of e... → NDCG@10: 0.8180 (Δ-0.1289)
  Persona: A seasoned software engineer working for a large t... → NDCG@10: 0.9060 (Δ-0.0409)
  Persona: A new graduate who recently launched an Android ap... → NDCG@10: 0.9675 (Δ0.0206)
  Persona: A conspiracy theorist and self-proclaimed tax expe... → NDCG@10: 0.9060 (Δ-0.0409)

Query 8: What options are available for a home loan with poor credit ...
Baseline NDCG@10: 0.2961
  Persona: Alicia is a 32-year-old single mother working as a... → NDCG@10: 0.1357 (Δ-0.1604)
  Persona: Raj is a 45-year-old financial advisor with over 2... → NDCG@10: 0.2961 (Δ0.0000)
  Persona: Samantha is a 28-year-old recent college graduate ... → NDCG@10: 0.1480 (Δ-0.1480)
  Persona: Tom is a 50-year-old real estate agent who believe... → NDCG@10: 0.4693 (Δ0.1732)

Query 9: How should I prepare for the next financial crisis?
Baseline NDCG@10: 0.0000
  Persona: A seasoned financial analyst at a large investment... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: Tom, a small business owner with a background in m... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: Emily, a recent graduate with a degree in economic... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: Derek, a conspiracy theorist and self-proclaimed e... → NDCG@10: 0.0000 (Δ0.0000)

Query 10: Can a car company refuse to give me a copy of my contract or...
Baseline NDCG@10: 0.6601
  Persona: A 35-year-old financial analyst working at a major... → NDCG@10: 0.5531 (Δ-0.1070)
  Persona: A 27-year-old recent graduate with a degree in bus... → NDCG@10: 0.5531 (Δ-0.1070)
  Persona: A 45-year-old auto industry consultant with over 2... → NDCG@10: 0.5531 (Δ-0.1070)
  Persona: A 50-year-old retired mechanic who believes that c... → NDCG@10: 0.5531 (Δ-0.1070)

Query 11: Is there any emprical research done on 'adding to a loser'
Baseline NDCG@10: 0.3869
  Persona: A financial analyst with over 10 years of experien... → NDCG@10: 0.6131 (Δ0.2263)
  Persona: A novice investor who has recently started learnin... → NDCG@10: 0.3869 (Δ0.0000)
  Persona: A behavioral economist conducting research on inve... → NDCG@10: 0.3066 (Δ-0.0803)
  Persona: A conspiracy theorist who believes that traditiona... → NDCG@10: 0.3869 (Δ0.0000)

Query 12: few question about debit credit and liabilities
Baseline NDCG@10: 0.0000
  Persona: A small business owner with a background in retail... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A finance student currently enrolled in an introdu... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A seasoned accountant with over a decade of experi... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A conspiracy theorist who believes that traditiona... → NDCG@10: 0.0000 (Δ0.0000)

Query 13: Cheapest way to “wire” money in an Australian bank account t...
Baseline NDCG@10: 0.5294
  Persona: A freelance graphic designer traveling through Sou... → NDCG@10: 0.5414 (Δ0.0120)
  Persona: An Australian expat living in Laos, working remote... → NDCG@10: 0.7757 (Δ0.2463)
  Persona: A financial advisor based in Australia who is curr... → NDCG@10: 0.7031 (Δ0.1736)
  Persona: A conspiracy theorist who believes that traditiona... → NDCG@10: 0.3904 (Δ-0.1391)

Query 14: What type of returns Vanguard is quoting?
Baseline NDCG@10: 1.0000
  Persona: A financial analyst at a mid-sized investment firm... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A retirement planner with a background in financia... → NDCG@10: 0.6309 (Δ-0.3691)
  Persona: A novice investor with a keen interest in personal... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: An anti-establishment activist who believes that l... → NDCG@10: 1.0000 (Δ0.0000)

Query 15: What is the formula for the Tesla Finance calculation?
Baseline NDCG@10: 0.0000
  Persona: A financial analyst at a renewable energy firm, sh... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A recent business school graduate exploring career... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A small business owner considering electric vehicl... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A traditional automotive industry veteran, he is s... → NDCG@10: 0.0000 (Δ0.0000)

Query 16: Prepaid Rent (Accrual Based Accounting)
Baseline NDCG@10: 1.0000
  Persona: Annelise is a junior accountant at a mid-sized fir... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: Raj is a property manager with over a decade of ex... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: Clara is an accounting professor at a university, ... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: Tom is a self-employed landlord who primarily oper... → NDCG@10: 1.0000 (Δ0.0000)

Query 17: US resident with Canadian income via T4A-NR
Baseline NDCG@10: 0.3333
  Persona: A Canadian tax consultant based in Toronto who spe... → NDCG@10: 0.3333 (Δ0.0000)
  Persona: An American expatriate living in Canada with a bac... → NDCG@10: 0.0000 (Δ-0.3333)
  Persona: A recent college graduate who has just started wor... → NDCG@10: 0.3869 (Δ0.0535)
  Persona: A self-proclaimed financial guru who believes that... → NDCG@10: 0.3869 (Δ0.0535)

Query 18: Thrift Saving Plan (TSP) Share Price Charts
Baseline NDCG@10: 0.9675
  Persona: A financial analyst working at an investment firm,... → NDCG@10: 1.0000 (Δ0.0325)
  Persona: A military veteran who has recently transitioned t... → NDCG@10: 0.9675 (Δ0.0000)
  Persona: A young tech entrepreneur who prefers unconvention... → NDCG@10: 1.0000 (Δ0.0325)
  Persona: A skeptical conspiracy theorist who believes tradi... → NDCG@10: 1.0000 (Δ0.0325)

Query 19: Pay off car loan entirely or leave $1 until the end of the l...
Baseline NDCG@10: 0.2240
  Persona: A mid-level financial analyst in their late 30s, E... → NDCG@10: 0.2832 (Δ0.0592)
  Persona: A single mother working as a nurse, Sarah is focus... → NDCG@10: 0.2337 (Δ0.0096)
  Persona: An entrepreneurial millennial who runs a small bus... → NDCG@10: 0.2489 (Δ0.0249)
  Persona: A retiree with no interest in managing finances, B... → NDCG@10: 0.1585 (Δ-0.0655)

Query 20: Why do stocks priced above $2.00 on the ASX sometimes move i...
Baseline NDCG@10: 0.0000
  Persona: A financial analyst working for an investment firm... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A retail investor with a background in engineering... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A finance professor at a university, he focuses on... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A conspiracy theorist who believes that stock mark... → NDCG@10: 0.0000 (Δ0.0000)

Query 21: What is the PEG ratio? How is the PEG ratio calculated? How ...
Baseline NDCG@10: 1.0000
  Persona: A financial analyst working for a hedge fund, she ... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A novice investor in his early 30s, he is eager to... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A finance professor at a university, he looks for ... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A conspiracy theorist who distrusts traditional fi... → NDCG@10: 1.0000 (Δ0.0000)

Query 22: How to change a large quantity of U.S. dollars into Euros?
Baseline NDCG@10: 0.3904
  Persona: A financial analyst working at an investment firm ... → NDCG@10: 0.5032 (Δ0.1128)
  Persona: An expatriate living in Europe who regularly trans... → NDCG@10: 0.2463 (Δ-0.1441)
  Persona: A small business owner planning to expand operatio... → NDCG@10: 0.3904 (Δ0.0000)
  Persona: A conspiracy theorist who believes that currency e... → NDCG@10: 0.3904 (Δ0.0000)

Query 23: Challenged an apparently bogus credit card charge, what happ...
Baseline NDCG@10: 0.0000
  Persona: A seasoned financial advisor with over 15 years of... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: Jason, a recent college graduate with a degree in ... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: Maria is a small business owner who frequently use... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: Tom, a conspiracy theorist and self-proclaimed fin... → NDCG@10: 0.0000 (Δ0.0000)

Query 24: Stock market vs. baseball card trading analogy
Baseline NDCG@10: 0.7654
  Persona: A financial analyst with over a decade of experien... → NDCG@10: 0.7654 (Δ0.0000)
  Persona: A high school economics teacher passionate about m... → NDCG@10: 0.7654 (Δ0.0000)
  Persona: A casual investor and baseball card collector with... → NDCG@10: 0.7654 (Δ0.0000)
  Persona: A retired baseball player who believes that the va... → NDCG@10: 0.7654 (Δ0.0000)

Query 25: How to donate to charity that will make a difference?
Baseline NDCG@10: 0.2021
  Persona: A philanthropic financial advisor with over a deca... → NDCG@10: 0.2346 (Δ0.0325)
  Persona: A recent college graduate passionate about social ... → NDCG@10: 0.2346 (Δ0.0325)
  Persona: A busy corporate executive who often donates to ch... → NDCG@10: 0.2346 (Δ0.0325)
  Persona: A skeptical journalist who often critiques charita... → NDCG@10: 0.2346 (Δ0.0325)

Query 26: Which U.S. online discount broker is the best value for mone...
Baseline NDCG@10: 0.0000
  Persona: A seasoned financial advisor with over 15 years of... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A college student majoring in finance who is relat... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A small business owner with limited experience in ... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A retiree who is skeptical of online trading platf... → NDCG@10: 0.1021 (Δ0.1021)

Query 27: Would it make sense to sell a stock, then repurchase it for ...
Baseline NDCG@10: 0.0000
  Persona: A financial analyst for a mid-sized investment fir... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A retail investor with a moderate understanding of... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A tax advisor specializing in investment strategie... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A conspiracy theorist who believes that stock mark... → NDCG@10: 0.0000 (Δ0.0000)

Query 28: Where are Bogleheadian World ETFs or Index funds?
Baseline NDCG@10: 0.2021
  Persona: Alicia is a certified financial planner with over ... → NDCG@10: 0.6714 (Δ0.4693)
  Persona: James is a recent college graduate and a finance e... → NDCG@10: 0.4441 (Δ0.2420)
  Persona: Maria is a data analyst in the tech sector with a ... → NDCG@10: 0.2961 (Δ0.0940)
  Persona: Tom is a vocal advocate for actively managed funds... → NDCG@10: 0.1357 (Δ-0.0665)

Query 29: Can the Delta be used to calculate the option premium given ...
Baseline NDCG@10: 0.2372
  Persona: A seasoned options trader with over a decade of ex... → NDCG@10: 0.2044 (Δ-0.0328)
  Persona: A finance student in their final year, this person... → NDCG@10: 0.2372 (Δ0.0000)
  Persona: A financial advisor working with clients intereste... → NDCG@10: 0.2641 (Δ0.0269)
  Persona: A stock market skeptic who prioritizes long-term i... → NDCG@10: 0.2184 (Δ-0.0188)

Query 30: Rules for SEP contributions in an LLC?
Baseline NDCG@10: 1.0000
  Persona: A small business owner in the tech startup space, ... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A financial advisor with ten years of experience, ... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A recent graduate entering the finance industry, h... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A skeptical journalist who frequently covers stori... → NDCG@10: 0.6309 (Δ-0.3691)

Query 31: Is there anything comparable to/resembling CNN's Fear and Gr...
Baseline NDCG@10: 0.1934
  Persona: A financial analyst at an investment firm, Sarah h... → NDCG@10: 0.5110 (Δ0.3175)
  Persona: A novice investor named James recently started tra... → NDCG@10: 0.4825 (Δ0.2890)
  Persona: Lisa is a behavioral economist researching the psy... → NDCG@10: 0.4825 (Δ0.2890)
  Persona: A conspiracy theorist, Mike believes that all fina... → NDCG@10: 0.1934 (Δ0.0000)

Query 32: UK sole trader who often buys products/services on behalf of...
Baseline NDCG@10: 1.0000
  Persona: A freelance graphic designer who often purchases s... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: An experienced accountant who specializes in small... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A newly established online retailer operating as a... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A small business owner who operates in a completel... → NDCG@10: 1.0000 (Δ0.0000)

Query 33: Are Exchange-Traded Funds (ETFs) less safe than regular mutu...
Baseline NDCG@10: 0.5912
  Persona: A financial analyst working for a major investment... → NDCG@10: 0.3869 (Δ-0.2044)
  Persona: A retiree with a modest investment portfolio, he i... → NDCG@10: 0.0000 (Δ-0.5912)
  Persona: A college student majoring in finance, he is condu... → NDCG@10: 0.6131 (Δ0.0219)
  Persona: A conspiracy theorist who believes the financial s... → NDCG@10: 0.2641 (Δ-0.3272)

Query 34: Freelancer: Should I start a second bank account?
Baseline NDCG@10: 1.0000
  Persona: A freelance graphic designer with several years of... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A newly transitioned freelance writer who recently... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A seasoned freelance consultant in the tech indust... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A part-time artist who primarily relies on social ... → NDCG@10: 1.0000 (Δ0.0000)

Query 35: Should I replace bonds in a passive investment strategy
Baseline NDCG@10: 0.5543
  Persona: A financial advisor with over a decade of experien... → NDCG@10: 0.5590 (Δ0.0047)
  Persona: A new investor in his late twenties, he works in t... → NDCG@10: 0.7157 (Δ0.1614)
  Persona: A retiree with a background in education, he is lo... → NDCG@10: 0.5805 (Δ0.0262)
  Persona: A conspiracy theorist and self-proclaimed financia... → NDCG@10: 0.6886 (Δ0.1343)

Query 36: Does it make sense to trade my GOOGL shares for GOOG and poc...
Baseline NDCG@10: 0.6714
  Persona: A seasoned financial analyst at a hedge fund, this... → NDCG@10: 0.6257 (Δ-0.0457)
  Persona: A novice retail investor who has recently started ... → NDCG@10: 0.4693 (Δ-0.2021)
  Persona: A technology sector analyst working for a financia... → NDCG@10: 0.6714 (Δ0.0000)
  Persona: A casual investor primarily focused on cryptocurre... → NDCG@10: 0.4693 (Δ-0.2021)

Query 37: Dealer Financing Fell Through on vehicle purchase: Scam?
Baseline NDCG@10: 0.6309
  Persona: A first-time car buyer in their late 20s working a... → NDCG@10: 0.5000 (Δ-0.1309)
  Persona: An automotive industry intern with a background in... → NDCG@10: 1.0000 (Δ0.3691)
  Persona: A seasoned financial advisor in their 50s, experie... → NDCG@10: 0.6309 (Δ0.0000)
  Persona: A car enthusiast and hobbyist blogger who dismisse... → NDCG@10: 0.5000 (Δ-0.1309)

Query 38: Short term parking of a large inheritance?
Baseline NDCG@10: 0.3392
  Persona: Anita is a financial advisor with over 10 years of... → NDCG@10: 0.3392 (Δ0.0000)
  Persona: James is a recent graduate with a degree in person... → NDCG@10: 0.3392 (Δ0.0000)
  Persona: Maria is a single mother and small business owner ... → NDCG@10: 0.3392 (Δ0.0000)
  Persona: Greg is a self-proclaimed investment guru who beli... → NDCG@10: 0.3392 (Δ0.0000)

Query 39: Should market based health insurance premiums be factored in...
Baseline NDCG@10: 0.8756
  Persona: A financial advisor with over a decade of experien... → NDCG@10: 0.8756 (Δ0.0000)
  Persona: A recent college graduate beginning their career i... → NDCG@10: 0.8928 (Δ0.0171)
  Persona: A stay-at-home parent who recently returned to the... → NDCG@10: 0.9268 (Δ0.0511)
  Persona: A conspiracy theorist who believes that the health... → NDCG@10: 0.9268 (Δ0.0511)

Query 40: Better to rent condo to daughter or put her on title?
Baseline NDCG@10: 0.8180
  Persona: A financial advisor with over a decade of experien... → NDCG@10: 0.8385 (Δ0.0206)
  Persona: A young entrepreneur who recently started a home-b... → NDCG@10: 0.7845 (Δ-0.0335)
  Persona: A retired individual with years of experience in r... → NDCG@10: 0.7845 (Δ-0.0335)
  Persona: A college student studying sociology, he is more i... → NDCG@10: 0.7929 (Δ-0.0251)

Query 41: When can you adjust for (and re-allow) a disallowed year-end...
Baseline NDCG@10: 0.3155
  Persona: A financial advisor specializing in tax strategies... → NDCG@10: 0.2891 (Δ-0.0264)
  Persona: An individual retail investor with a hobbyist inte... → NDCG@10: 0.3333 (Δ0.0179)
  Persona: A tax accountant with over a decade of experience,... → NDCG@10: 0.2891 (Δ-0.0264)
  Persona: A casual investor who believes that tax rules are ... → NDCG@10: 0.3155 (Δ0.0000)

Query 42: Would the effects of an anticipated default by a nation be m...
Baseline NDCG@10: 0.6131
  Persona: A financial analyst at a major investment firm, Sa... → NDCG@10: 0.6131 (Δ0.0000)
  Persona: A government policy advisor with experience in int... → NDCG@10: 0.7904 (Δ0.1772)
  Persona: An economics student, Emily is researching the the... → NDCG@10: 0.6131 (Δ0.0000)
  Persona: A conspiracy theorist blogger, Jake dismisses main... → NDCG@10: 0.6131 (Δ0.0000)

Query 43: Selling mutual fund and buying equivalent ETF: Can I 1031 ex...
Baseline NDCG@10: 1.0000
  Persona: A middle-aged financial advisor with over 15 years... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A recent college graduate working at a fintech sta... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A seasoned tax accountant with a focus on real est... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A retiree who has recently started managing their ... → NDCG@10: 1.0000 (Δ0.0000)

Query 44: What does Chapter 11 Bankruptcy mean to an investor holding ...
Baseline NDCG@10: 0.8175
  Persona: A financial analyst at a hedge fund, with five yea... → NDCG@10: 0.8772 (Δ0.0597)
  Persona: A small business owner who has invested personal s... → NDCG@10: 0.8175 (Δ0.0000)
  Persona: An accounting student currently studying corporate... → NDCG@10: 0.8772 (Δ0.0597)
  Persona: A conspiracy theorist who believes bankruptcy laws... → NDCG@10: 0.8772 (Δ0.0597)

Query 45: Privacy preferences on creditworthiness data
Baseline NDCG@10: 0.5000
  Persona: A financial analyst at a mid-sized bank, she is ke... → NDCG@10: 0.5000 (Δ0.0000)
  Persona: A privacy advocate and legal consultant, he is int... → NDCG@10: 0.3869 (Δ-0.1131)
  Persona: A small business owner with a background in entrep... → NDCG@10: 0.6309 (Δ0.1309)
  Persona: A tech entrepreneur who believes that data should ... → NDCG@10: 0.3333 (Δ-0.1667)

Query 46: How to exclude stock from mutual fund
Baseline NDCG@10: 0.6257
  Persona: A financial advisor with over ten years of experie... → NDCG@10: 0.6364 (Δ0.0107)
  Persona: A novice investor who recently started managing th... → NDCG@10: 0.6173 (Δ-0.0084)
  Persona: A compliance officer at a major investment firm, h... → NDCG@10: 0.6173 (Δ-0.0084)
  Persona: A conspiracy theorist who believes that mutual fun... → NDCG@10: 0.6257 (Δ0.0000)

Query 47: Investing in commodities, pros and cons?
Baseline NDCG@10: 0.8503
  Persona: A seasoned financial analyst working in a hedge fu... → NDCG@10: 0.9197 (Δ0.0694)
  Persona: Mark is a small business owner who has recently be... → NDCG@10: 0.9197 (Δ0.0694)
  Persona: Emily, a graduate student specializing in environm... → NDCG@10: 0.9197 (Δ0.0694)
  Persona: John, a self-proclaimed finance guru who primarily... → NDCG@10: 0.8772 (Δ0.0269)

Query 48: Online tool to connect to my bank account and tell me what I...
Baseline NDCG@10: 0.0000
  Persona: Arianna is a 28-year-old financial analyst working... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: James is a 45-year-old small business owner who ju... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: Nina is a 32-year-old graduate student studying so... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: Mark is a 50-year-old retired military veteran who... → NDCG@10: 0.0000 (Δ0.0000)

Query 49: Freelance site with lowest commission fees?
Baseline NDCG@10: 0.0000
  Persona: A seasoned freelance graphic designer working prim... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A recent college graduate entering the freelance w... → NDCG@10: 0.3010 (Δ0.3010)
  Persona: An experienced project manager in the tech industr... → NDCG@10: 0.0000 (Δ0.0000)
  Persona: A corporate employee who values job security and s... → NDCG@10: 0.5000 (Δ0.5000)

Query 50: How does spot-futures arbitrage work in the gold market?
Baseline NDCG@10: 1.0000
  Persona: A seasoned commodities trader specializing in prec... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: An economics student pursuing a master's degree, t... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A financial analyst at a major investment firm, th... → NDCG@10: 1.0000 (Δ0.0000)
  Persona: A retail jewelry store owner who is concerned abou... → NDCG@10: 1.0000 (Δ0.0000)

============================================================
SUMMARY
Average baseline NDCG@10: 0.5195
Average improvement: 0.0106
Positive improvements: 65/200 (32.5%)
