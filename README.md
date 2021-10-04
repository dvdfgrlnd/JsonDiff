# JsonDiff
# Work in progress...

## Description

A tool to find differences in json data. 

Test [here](https://dvdfgrlnd.github.io/JsonDiff/web_output/index.html).

### Example


<table>
<tr>
<th> Json 1 </th>
<th> Json 2 </th>
<th> Result </th>
</tr>
<tr>
<td>

```json
{
   "f1":"v1",
   "f2":[
      {
         "f3":"v3",
         "f4":[
            1,
            2,
            3
         ]
      },
      {
         "f3":"v3",
         "f4":[
            1,
            2,
            3,
            4
         ]
      }
   ]
}
```

</td>
<td>

```json
{
   "f1":true,
   "f2":[
      {
         "f3":"v3",
         "f4":[
            1,
            2,
            3
         ]
      },
      {
         "f3":"v3",
         "f4":[
            1,
            2,
            3
         ]
      }
   ]
}
```

</td>
  
<td>
  
  ```
  {
    f1: ###
    +++"v1"+++ ,
    ---true---
    ###,
    f2: [
            {
                f3: "v3",
                    f4: [
                            1,
                            2,
                            3,
                        ],
            },
            {
                f3: +++"v3"+++ ,
                f4: [
                    +++1+++ ,
                    +++2+++ ,
                    +++3+++ ,
                    +++4+++ ,
                ],
            },
            {
                f3: ---"v3"--- ,
                f4: [
                    ---1--- ,
                    ---2--- ,
                    ---3--- ,
                ],
            },
        ],
}
  ```
  
  </td>
  
</tr>
</table>


