An important aspect of adjusting the parameters of the NN model is the evaluation of the resulting output. With music in particular this can be a pain point due to its complex nature and the subjectivety that is often part in determining its value. This subjectivety presents a great challenge in regards to automating the evaluation of the model output and by extension the process of improving the model.

To alleviate this we must explore means of quantitatively analysing the model output; to which end we must have insight into what good music, good or alternatively what makes music bad.

# Repetition & Patterns
Common among humans is the propensity to look for patterns in everything we do and experience. This of course extends to music as well. In music we find repeating and recurring elements both at micro and macro levels (chord progressions; melodies; structural divisions: chorus, verse). These patterns in music generally play a part in making music feel pleasent to listen to, as well being one of the the main driving forces when it comes making a song catchy. (Don't quote me on this last part XD)

## An intuition
While knowledge of what kinds of repetition occur, and the frequency at which they do so, is unlikely to tell whether a song is good, it may prove useful when it comes to identifying songs that are bad. When considering repetition in music, in a somewhat simplistic manner, one may intuitively reason that high levels repetition could result in songs that may be considered boring. Counter to that, a song with no repetition at all may be described as noisy or unpleasent. 

# Bibliography
```tex
@phdthesis{repetition-struct-analysis,
  author      = {Nanzhu Jiang},
  title       = {Repetition-based Structure Analysis of Music Recordings},
  type        = {doctoralthesis},
  school      = {Friedrich-Alexander-Universit{\"a}t Erlangen-N{\"u}rnberg (FAU)},
  year        = {2015},
}

@article{ideas-auto-eval,
  author      = {Freitas, A. and Guimarães, Frederico and Barbosa, Rogério},
  year        = {2012},
  month       = {01},
  pages       = {},
  title       = {Ideas in automatic evaluation methods for melodies in algorithmic   composition},
  journal     = {Proceedings of the 9th Sound and Music Computing Conference, SMC 2012}
}

% Original paper from which the Google MAESTRO dataset originates
@inproceedings{maestro,
  title       = {Enabling Factorized Piano Music Modeling and Generation with the {MAESTRO} Dataset},
  author      = {Curtis Hawthorne and Andriy Stasyuk and Adam Roberts and Ian Simon and Cheng-Zhi Anna Huang and Sander Dieleman and Erich Elsen and Jesse Engel and Douglas Eck},
  booktitle   = {International Conference on Learning Representations},
  year        = {2019},
  url         = {https://openreview.net/forum?id=r1lYRjC9F7},
}
```