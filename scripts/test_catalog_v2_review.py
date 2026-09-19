import unittest
from catalog_v2_review import compare, price


class ComparisonTests(unittest.TestCase):
    def source(self, name='Medicine 10 MG 20 TABS', amount='10.0'):
        return [name, 'دواء', 'Ingredient', 'Maker', 'Class', 'ORAL.SOLID', amount]

    def product(self, fields=None):
        return {'id': 'existing', 'package_id': 'package', 'source_row': 1,
                'fields': (fields or self.source())[:6], 'price_minor': 1000}

    def run_compare(self, rows, products=None, malformed=()):
        old = {'number': 1, 'fields': self.source(), 'fingerprint': 'historical'}
        return compare(rows, [self.product()] if products is None else products, {1: old}, malformed)

    def test_padding_is_unchanged_but_raw_fingerprint_remains_distinct(self):
        a = self.run_compare([self.source()])[0]
        b = self.run_compare([[' ' + v + ' ' for v in self.source()]])[0]
        self.assertEqual(b['classification'], 'UNCHANGED')
        self.assertNotEqual(a['sourceRowFingerprint'], b['sourceRowFingerprint'])
        self.assertEqual(a['logicalFingerprint'], b['logicalFingerprint'])

    def test_price_is_exact_and_future_append_is_only_a_proposal(self):
        result = self.run_compare([self.source(amount='11.25')])[0]
        self.assertEqual(result['classification'], 'CHANGED')
        self.assertEqual(result['changes']['price_egp']['v2Minor'], 1125)
        self.assertFalse(result['applied'])
        self.assertIsNone(price('4.415'))
        self.assertEqual(self.run_compare([self.source(amount='4.415')])[0]['classification'], 'REVIEW_REQUIRED')

    def test_duplicates_never_merge_and_different_strength_requires_review(self):
        self.assertTrue(all(r['classification'] == 'POSSIBLE_DUPLICATE' for r in self.run_compare([self.source(), self.source()])))
        self.assertEqual(self.run_compare([self.source('Medicine 20 MG 20 TABS')])[0]['classification'], 'REVIEW_REQUIRED')

    def test_new_requires_no_candidate_and_complete_data(self):
        self.assertEqual(self.run_compare([self.source()], [])[0]['classification'], 'NEW')
        incomplete = self.source()
        incomplete[2] = ''
        self.assertEqual(self.run_compare([incomplete], [])[0]['classification'], 'REVIEW_REQUIRED')

    def test_lookup_display_variants_are_not_local_changes(self):
        product = self.product()
        product['fields'][3] = 'maker'
        result = self.run_compare([self.source()], [product])[0]
        self.assertEqual(result['classification'], 'UNCHANGED')
        self.assertEqual(result['localChangesSinceV1'], [])

    def test_malformed_and_ambiguous_quotes_are_held(self):
        self.assertEqual(self.run_compare([self.source()], malformed=[1])[0]['classification'], 'REVIEW_REQUIRED')
        bad = self.source()
        bad[3] = '"Maker"'
        self.assertEqual(self.run_compare([bad])[0]['classification'], 'REVIEW_REQUIRED')
        self.assertEqual(self.run_compare([['fragment']])[0]['classification'], 'REVIEW_REQUIRED')


if __name__ == '__main__':
    unittest.main()
